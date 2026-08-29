<template>
  <div id="app-wrapper" @mousedown.self="handleGlobalClick">
    <div class="app-shell">
      <header class="titlebar" data-tauri-drag-region @mousedown="startDrag">
        <div class="brand" data-tauri-drag-region>
          <img :src="appLogoUrl" alt="" />
          <span>any2bibtex</span>
        </div>
        <div class="titlebar-actions" @mousedown.stop>
          <button
            class="icon-button update-button"
            :class="{ active: activePanel === 'update' }"
            type="button"
            aria-label="Check for updates"
            title="Check for updates"
            @click="openUpdatePanel"
          >
            <RefreshCw
              :class="{ spin: checkingForUpdate }"
              :size="16"
              :stroke-width="1.8"
              aria-hidden="true"
            />
            <span v-if="availableUpdate" class="notification-dot"></span>
          </button>
          <button
            class="icon-button"
            type="button"
            :aria-label="theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
            :title="theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
            @click="toggleTheme"
          >
            <Sun v-if="theme === 'dark'" :size="16" :stroke-width="1.8" aria-hidden="true" />
            <Moon v-else :size="16" :stroke-width="1.8" aria-hidden="true" />
          </button>
          <button
            class="icon-button"
            :class="{ active: activePanel === 'settings' }"
            type="button"
            aria-label="Semantic Scholar settings"
            title="Semantic Scholar settings"
            @click="openApiKeyPanel"
          >
            <Settings :size="16" :stroke-width="1.8" aria-hidden="true" />
          </button>
        </div>
      </header>

      <main class="workspace">
        <template v-if="activePanel === 'search'">
          <SearchBar
            v-model="query"
            :input-type="detectedInputType"
            :loading="loading"
            @search="handleSearch"
            @escape="handleEscape"
          />

          <ResultCard
            :loading="loading"
            :loading-message="loadingMessage"
            :loading-detail="loadingDetail"
            :error="error"
            :error-hint="errorHint"
            :bibtex="formattedBibtex"
            :input-type="inputType"
            :copied="copied"
            :api-key-configured="apiKeyConfigured"
            :active-input-type="activeInputType"
            @copy="copyBibtex"
            @configure-api-key="openApiKeyPanel"
          />
        </template>

        <ApiKeyPanel
          v-else-if="activePanel === 'settings'"
          :api-key-configured="apiKeyConfigured"
          @close="closePanel"
          @saved="handleApiKeySaved"
          @error="handleApiKeyError"
        />

        <UpdatePanel
          v-else
          :initial-update="availableUpdate"
          @close="closeUpdatePanel"
          @update-found="handleUpdateFound"
          @busy-change="updateBusy = $event"
        />
      </main>

      <footer v-if="activePanel === 'search'" class="statusbar">
        <div class="status-copy">
          <span class="status-dot" :data-state="statusState"></span>
          <span>{{ statusText }}</span>
        </div>
        <span v-if="apiKeyConfigured" class="source-status">
          <KeyRound :size="12" :stroke-width="1.8" aria-hidden="true" />
          Semantic Scholar
        </span>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { KeyRound, Moon, RefreshCw, Settings, Sun } from "@lucide/vue";
import type { Update } from "@tauri-apps/plugin-updater";
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from "vue";
import appLogoUrl from "../assets/logo.png";
import ApiKeyPanel from "./components/ApiKeyPanel.vue";
import ResultCard from "./components/ResultCard.vue";
import SearchBar from "./components/SearchBar.vue";
import UpdatePanel from "./components/UpdatePanel.vue";
import {
  copyToClipboard,
  getAppTheme,
  getSemanticScholarConfig,
  hideWindow,
  onOpenSettingsRequested,
  onOpenUpdateRequested,
  onThemeChanged,
  resolveQuery,
  setAppTheme,
  startWindowDrag,
} from "./services/desktop";
import {
  checkForAppUpdate,
  hasCompletedUpdateMarker,
} from "./services/updater";
import { formatBibtex } from "./utils/bibtex";

type AppPanel = "search" | "settings" | "update";

const query = ref("");
const rawBibtex = ref("");
const inputType = ref("");
const activeInputType = ref("");
const loading = ref(false);
const error = ref("");
const copied = ref(false);
const apiKeyConfigured = ref(false);
const activePanel = ref<AppPanel>("search");
const theme = ref<"dark" | "light">("dark");
const availableUpdate = shallowRef<Update | null>(null);
const checkingForUpdate = ref(false);
const updateBusy = ref(false);
let removeThemeListener: (() => void) | null = null;
let removeOpenSettingsListener: (() => void) | null = null;
let removeOpenUpdateListener: (() => void) | null = null;
let updateCheckTimer: ReturnType<typeof setTimeout> | null = null;

const formattedBibtex = computed(() => formatBibtex(rawBibtex.value));
const detectedInputType = computed(() => identifyInputType(query.value));

const loadingMessage = computed(() => {
  if (activeInputType.value === "title") return "Searching Semantic Scholar";
  if (activeInputType.value === "doi") return "Resolving DOI metadata";
  if (activeInputType.value === "arxiv") return "Resolving arXiv metadata";
  return "Resolving paper";
});

const loadingDetail = computed(() => {
  if (activeInputType.value !== "title" || apiKeyConfigured.value) return "";
  return "Shared requests can be slower during busy periods.";
});

const errorHint = computed(() => {
  if (!error.value) return "";
  const lowered = error.value.toLowerCase();

  if (activeInputType.value === "title" && lowered.includes("rate limit")) {
    return apiKeyConfigured.value
      ? "Semantic Scholar is temporarily limiting requests."
      : "Adding an API key can make title search more reliable.";
  }
  if (activeInputType.value === "title" && lowered.includes("reliable title match")) {
    return "Try the exact paper title, DOI, or arXiv ID.";
  }
  return "";
});

const statusState = computed(() => {
  if (loading.value || checkingForUpdate.value) return "working";
  if (error.value) return "error";
  if (availableUpdate.value) return "update";
  if (formattedBibtex.value) return "success";
  return "ready";
});

const statusText = computed(() => {
  if (loading.value) return loadingMessage.value;
  if (checkingForUpdate.value) return "Checking for updates";
  if (error.value) return "Needs attention";
  if (availableUpdate.value) return `Update v${availableUpdate.value.version} available`;
  if (formattedBibtex.value) return `${inputType.value.toUpperCase()} resolved`;
  return "Ready";
});

watch(query, (newValue) => {
  if (newValue.trim()) return;
  rawBibtex.value = "";
  error.value = "";
  inputType.value = "";
  activeInputType.value = "";
});

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  void loadApiKeyConfig();
  void loadThemeConfig();

  if (hasCompletedUpdateMarker()) {
    activePanel.value = "update";
  } else {
    updateCheckTimer = setTimeout(() => {
      void checkForUpdatesSilently();
    }, 1200);
  }

  onThemeChanged(applyTheme)
    .then((unlisten) => {
      removeThemeListener = unlisten;
    })
    .catch((themeError) => {
      console.warn("Theme events are unavailable:", themeError);
    });

  onOpenSettingsRequested(openApiKeyPanel)
    .then((unlisten) => {
      removeOpenSettingsListener = unlisten;
    })
    .catch((listenerError) => {
      console.warn("Settings menu events are unavailable:", listenerError);
    });

  onOpenUpdateRequested(openUpdatePanel)
    .then((unlisten) => {
      removeOpenUpdateListener = unlisten;
    })
    .catch((listenerError) => {
      console.warn("Update menu events are unavailable:", listenerError);
    });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
  removeThemeListener?.();
  removeOpenSettingsListener?.();
  removeOpenUpdateListener?.();
  if (updateCheckTimer) clearTimeout(updateCheckTimer);
});

function identifyInputType(value: string): string {
  const trimmed = value.trim();
  const doiPattern = /^10\.\d{4,}\/[^\s]+$/;
  const arxivPattern = /^(\d{4}\.\d{4,5})(v\d+)?$|^[a-z-]+\/\d{7}$/i;

  if (doiPattern.test(trimmed) || trimmed.includes("doi.org/")) return "doi";
  if (arxivPattern.test(trimmed) || trimmed.includes("arxiv.org")) return "arxiv";
  return "title";
}

function formatResolveError(message: string, detectedType: string): string {
  const lowered = message.toLowerCase();

  if (detectedType === "title" && lowered.includes("rate limit")) {
    return "Semantic Scholar is busy right now.";
  }
  if (detectedType === "title" && lowered.includes("no title matches")) {
    return "No paper matched this title.";
  }
  if (detectedType === "title" && lowered.includes("confident title match")) {
    return "No reliable title match was found.";
  }
  if (detectedType === "title") {
    return "Title search failed.";
  }
  return message || "The paper could not be resolved.";
}

async function handleSearch() {
  const normalizedQuery = query.value.trim();
  if (!normalizedQuery || loading.value) return;

  activeInputType.value = identifyInputType(normalizedQuery);
  loading.value = true;
  error.value = "";
  rawBibtex.value = "";
  copied.value = false;

  try {
    const data = await resolveQuery(normalizedQuery);
    if (data.success && data.bibtex) {
      rawBibtex.value = data.bibtex;
      inputType.value = data.type;
    } else {
      error.value = formatResolveError(data.error || "", activeInputType.value);
    }
  } catch (resolveError) {
    console.error("Resolver failed:", resolveError);
    error.value = "The resolver could not connect.";
  } finally {
    loading.value = false;
  }
}

async function copyBibtex() {
  if (!formattedBibtex.value) return;

  try {
    await copyToClipboard(formattedBibtex.value);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 1800);
  } catch (copyError) {
    console.error("Copy failed:", copyError);
    error.value = "BibTeX could not be copied.";
  }
}

async function loadApiKeyConfig() {
  try {
    const config = await getSemanticScholarConfig();
    apiKeyConfigured.value = Boolean(config?.hasApiKey);
  } catch (configError) {
    console.warn("Semantic Scholar settings are unavailable:", configError);
  }
}

async function loadThemeConfig() {
  try {
    applyTheme(await getAppTheme());
  } catch (themeError) {
    console.warn("Theme settings are unavailable:", themeError);
    applyTheme("dark");
  }
}

async function toggleTheme() {
  const previousTheme = theme.value;
  const nextTheme = previousTheme === "dark" ? "light" : "dark";
  applyTheme(nextTheme);

  try {
    applyTheme(await setAppTheme(nextTheme));
  } catch (themeError) {
    console.warn("Failed to save theme:", themeError);
  }
}

async function checkForUpdatesSilently() {
  if (checkingForUpdate.value || availableUpdate.value) return;
  checkingForUpdate.value = true;

  try {
    const update = await checkForAppUpdate();
    if (update) {
      availableUpdate.value = update;
      activePanel.value = "update";
    }
  } catch (updateError) {
    console.warn("Automatic update check failed:", updateError);
  } finally {
    checkingForUpdate.value = false;
  }
}

function applyTheme(nextTheme: "dark" | "light") {
  theme.value = nextTheme === "light" ? "light" : "dark";
  document.documentElement.dataset.theme = theme.value;
}

function openApiKeyPanel() {
  if (updateBusy.value) return;
  activePanel.value = "settings";
}

function openUpdatePanel() {
  activePanel.value = "update";
}

function closePanel() {
  activePanel.value = "search";
}

function closeUpdatePanel() {
  if (!updateBusy.value) closePanel();
}

function handleUpdateFound(update: Update | null) {
  availableUpdate.value = update;
}

function handleApiKeySaved(hasApiKey: boolean) {
  apiKeyConfigured.value = hasApiKey;
}

function handleApiKeyError(message: string) {
  error.value = message;
}

function handleGlobalClick() {
  handleEscape();
}

function handleEscape() {
  if (activePanel.value !== "search") {
    if (!updateBusy.value) closePanel();
    return;
  }
  void hideWindow();
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key !== "Escape") return;
  event.preventDefault();
  handleEscape();
}

function startDrag(event: MouseEvent) {
  if (event.button !== 0) return;
  startWindowDrag().catch((dragError) => {
    console.warn("Window dragging is unavailable:", dragError);
  });
}
</script>

<style>
#app-wrapper {
  display: flex;
  width: 100vw;
  height: 100vh;
  align-items: center;
  justify-content: center;
  padding: 10px;
  background: transparent;
}

.app-shell {
  display: flex;
  width: 100%;
  height: min(100%, 460px);
  min-height: 0;
  max-width: 664px;
  max-height: 460px;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border-soft);
  border-radius: 10px;
  background: var(--app-bg);
  box-shadow: var(--shadow);
}

.titlebar {
  display: flex;
  min-height: 44px;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px 0 13px;
  border-bottom: 1px solid var(--border-soft);
  background: var(--surface-bg);
  cursor: grab;
  -webkit-app-region: drag;
}

.titlebar:active {
  cursor: grabbing;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 650;
  user-select: none;
}

.brand img {
  width: 24px;
  height: 24px;
  border-radius: 5px;
}

.titlebar-actions {
  display: flex;
  align-items: center;
  gap: 3px;
  -webkit-app-region: no-drag;
}

.icon-button {
  position: relative;
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-subtle);
  cursor: pointer;
  transition:
    border-color 140ms ease,
    background-color 140ms ease,
    color 140ms ease,
    transform 140ms ease;
}

.icon-button:hover,
.icon-button.active {
  border-color: var(--border-soft);
  background: var(--surface-raised);
  color: var(--text-main);
}

.icon-button:active {
  transform: scale(0.95);
}

.notification-dot {
  position: absolute;
  top: 5px;
  right: 5px;
  width: 6px;
  height: 6px;
  border: 1px solid var(--surface-bg);
  border-radius: 50%;
  background: var(--warning);
}

.workspace {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
}

.statusbar {
  display: flex;
  min-height: 34px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 14px;
  border-top: 1px solid var(--border-soft);
  background: var(--surface-bg);
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 550;
}

.status-copy,
.source-status {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 6px;
}

.status-copy span:last-child {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-dot {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: var(--text-subtle);
}

.status-dot[data-state="working"] {
  background: var(--warning);
}

.status-dot[data-state="error"] {
  background: var(--danger);
}

.status-dot[data-state="success"] {
  background: var(--success);
}

.status-dot[data-state="update"] {
  background: var(--accent);
}

.source-status {
  flex: 0 0 auto;
  color: var(--text-muted);
}

.spin {
  animation: app-spin 800ms linear infinite;
}

@keyframes app-spin {
  to {
    transform: rotate(360deg);
  }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    scroll-behavior: auto !important;
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
