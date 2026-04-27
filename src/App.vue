<template>
  <div id="app-wrapper" @mousedown.self="handleGlobalClick">
    <div class="spotlight-container">
      <button
        class="theme-toggle"
        :aria-label="theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
        :title="theme === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'"
        @click="toggleTheme"
      >
        {{ theme === "dark" ? "☀" : "☾" }}
      </button>

      <SearchBar
        v-model="query"
        @search="handleSearch"
        @escape="handleEscape"
      />

      <ResultCard
        :loading="loading"
        :loadingMessage="loadingMessage"
        :loadingDetail="loadingDetail"
        :error="error"
        :errorHint="errorHint"
        :bibtex="formattedBibtex"
        :inputType="inputType"
        :copied="copied"
        :apiKeyConfigured="apiKeyConfigured"
        :activeInputType="activeInputType"
        @copy="copyBibtex"
        @configureApiKey="openApiKeyPanel"
      />

      <ApiKeyPanel
        v-if="showApiKeyPanel"
        :apiKeyConfigured="apiKeyConfigured"
        @close="closeApiKeyPanel"
        @saved="handleApiKeySaved"
        @error="handleApiKeyError"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, Ref, watch } from "vue";
import ApiKeyPanel from "./components/ApiKeyPanel.vue";
import SearchBar from "./components/SearchBar.vue";
import ResultCard from "./components/ResultCard.vue";
import { formatBibtex } from "./utils/bibtex";

const API_BASE = "http://127.0.0.1:8765";

interface ResolveResponse {
  success: boolean;
  type: string;
  bibtex: string | null;
  error: string | null;
}

const query: Ref<string> = ref("");
const rawBibtex: Ref<string> = ref("");
const inputType: Ref<string> = ref("");
const activeInputType: Ref<string> = ref("");
const loading: Ref<boolean> = ref(false);
const error: Ref<string> = ref("");
const copied: Ref<boolean> = ref(false);
const apiKeyConfigured: Ref<boolean> = ref(false);
const showApiKeyPanel: Ref<boolean> = ref(false);
const theme: Ref<"dark" | "light"> = ref("dark");
let removeThemeListener: (() => void) | null = null;

const formattedBibtex = computed(() => formatBibtex(rawBibtex.value));
const loadingMessage = computed(() => {
  if (activeInputType.value === "title") {
    return "Searching Semantic Scholar for the best title match...";
  }
  if (activeInputType.value === "doi") {
    return "Resolving DOI metadata...";
  }
  if (activeInputType.value === "arxiv") {
    return "Resolving arXiv metadata...";
  }
  return "Resolving...";
});

const loadingDetail = computed(() => {
  if (activeInputType.value === "title") {
    return apiKeyConfigured.value
      ? "Using your configured Semantic Scholar API key for title search."
      : "Without a Semantic Scholar API key, title search uses shared rate limits.";
  }
  return "";
});

const errorHint = computed(() => {
  if (!error.value) return "";
  if (
    activeInputType.value === "title" &&
    error.value.toLowerCase().includes("rate limit")
  ) {
    return apiKeyConfigured.value
      ? "Your Semantic Scholar API key is configured, but the service is still throttling requests right now."
      : "Title search is using Semantic Scholar. Without an API key, requests may hit shared rate limits during busy periods.";
  }
  if (
    activeInputType.value === "title" &&
    error.value.toLowerCase().includes("confident title match")
  ) {
    return "Try a more exact paper title, or use DOI/arXiv ID for a precise lookup.";
  }
  return "";
});

function formatResolveError(message: string, detectedType: string): string {
  const lowered = message.toLowerCase();

  if (detectedType === "title" && lowered.includes("rate limit")) {
    return "Semantic Scholar is busy right now. Title search hit a shared rate limit.";
  }
  if (detectedType === "title" && lowered.includes("no title matches")) {
    return "No matching paper was found for this title.";
  }
  if (detectedType === "title" && lowered.includes("confident title match")) {
    return "No reliable title match was found.";
  }
  if (detectedType === "title") {
    return "Title search failed. Please try a more exact title, or use a DOI/arXiv ID.";
  }
  return message || "Failed to resolve. Please check your input.";
}

// Clear results when query is empty
watch(query, (newVal) => {
  if (!newVal.trim()) {
    rawBibtex.value = "";
    error.value = "";
    inputType.value = "";
    activeInputType.value = "";
  }
});

function identifyInputType(query: string): string {
  const trimmed = query.trim();
  const doiPattern = /^10\.\d{4,}\/[^\s]+$/;
  const arxivPattern =
    /^(\d{4}\.\d{4,5})(v\d+)?$|^[a-z-]+\/\d{7}$/i;

  if (doiPattern.test(trimmed) || trimmed.includes("doi.org/")) {
    return "doi";
  }
  if (arxivPattern.test(trimmed) || trimmed.includes("arxiv.org")) {
    return "arxiv";
  }
  return "title";
}

function handleGlobalClick() {
  handleEscape();
}

function handleEscape() {
  if (showApiKeyPanel.value) {
    closeApiKeyPanel();
    return;
  }
  const api = (window as any).electronAPI;
  if (api?.hideWindow) {
    api.hideWindow();
  }
}

// Global key handler (Fallback, primary is in main.js)
function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    handleEscape();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  loadApiKeyConfig();
  loadThemeConfig();
  removeThemeListener = window.electronAPI?.onThemeChanged?.((nextTheme) => {
    applyTheme(nextTheme);
  }) ?? null;
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
  removeThemeListener?.();
});

async function handleSearch() {
  const q = query.value.trim();
  if (!q) return;

  activeInputType.value = identifyInputType(q);
  loading.value = true;
  error.value = "";
  rawBibtex.value = "";
  copied.value = false;

  try {
    const response = await fetch(
      `${API_BASE}/resolve?q=${encodeURIComponent(q)}`,
    );
    const data: ResolveResponse = await response.json();

    if (data.success && data.bibtex) {
      rawBibtex.value = data.bibtex;
      inputType.value = data.type;
    } else {
      error.value = formatResolveError(
        data.error || "",
        activeInputType.value,
      );
    }
  } catch (err) {
    error.value =
      "Cannot connect to backend. Please ensure Python server is running.";
    console.error(err);
  } finally {
    loading.value = false;
  }
}

async function copyBibtex() {
  if (!formattedBibtex.value) return;

  try {
    const api = (window as any).electronAPI;
    // Copy the FORMATTED bibtex to be consistent with what the user sees
    if (api?.copyToClipboard) {
      await api.copyToClipboard(formattedBibtex.value);
    } else {
      await navigator.clipboard.writeText(formattedBibtex.value);
    }
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (err) {
    console.error("Copy failed:", err);
  }
}

async function loadApiKeyConfig() {
  try {
    const api = (window as any).electronAPI;
    if (!api?.getSemanticScholarConfig) return;
    const config = await api.getSemanticScholarConfig();
    apiKeyConfigured.value = Boolean(config?.hasApiKey);
  } catch (err) {
    console.error("Failed to load Semantic Scholar config:", err);
  }
}

function openApiKeyPanel() {
  showApiKeyPanel.value = true;
}

function closeApiKeyPanel() {
  showApiKeyPanel.value = false;
}

function handleApiKeySaved(hasApiKey: boolean) {
  apiKeyConfigured.value = hasApiKey;
}

function handleApiKeyError(message: string) {
  error.value = message;
}

function applyTheme(nextTheme: "dark" | "light") {
  theme.value = nextTheme;
  document.documentElement.dataset.theme = nextTheme;
}

async function loadThemeConfig() {
  try {
    const nextTheme = await window.electronAPI?.getAppTheme?.();
    applyTheme(nextTheme === "light" ? "light" : "dark");
  } catch (err) {
    console.error("Failed to load app theme:", err);
    applyTheme("dark");
  }
}

async function toggleTheme() {
  const nextTheme = theme.value === "dark" ? "light" : "dark";
  applyTheme(nextTheme);
  try {
    const savedTheme = await window.electronAPI?.setAppTheme?.(nextTheme);
    if (savedTheme) {
      applyTheme(savedTheme);
    }
  } catch (err) {
    console.error("Failed to save app theme:", err);
  }
}
</script>

<style>
/* Global styles wrapper */
#app-wrapper {
  width: 100vw;
  height: 100vh;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.001);
}

.spotlight-container {
  position: relative;
  width: 100%;
  max-width: 664px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--app-bg);
  border-radius: 16px;
}

.theme-toggle {
  position: absolute;
  top: 8px;
  right: 12px;
  z-index: 4;
  width: 30px;
  height: 30px;
  border: 1px solid var(--border-soft);
  border-radius: 50%;
  background: var(--control-bg);
  color: var(--text-main);
  cursor: pointer;
  font-size: 15px;
  line-height: 1;
  -webkit-app-region: no-drag;
}

.theme-toggle:hover {
  border-color: var(--accent);
}
</style>
