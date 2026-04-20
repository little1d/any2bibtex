<template>
  <div id="app-wrapper" @mousedown.self="handleGlobalClick">
    <div class="spotlight-container">
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

      <div
        v-if="showApiKeyPanel"
        class="settings-panel"
      >
        <div class="settings-panel-header">
          <div>
            <div class="settings-title">Semantic Scholar API Key</div>
            <div class="settings-copy">
              Title search works without a key, but anonymous requests use shared rate limits.
              Adding your own key makes title search more reliable.
            </div>
          </div>
          <button class="settings-close" @click="closeApiKeyPanel">Close</button>
        </div>

        <div class="settings-badge-row">
          <span class="settings-badge">Optional</span>
          <span class="settings-badge settings-badge-muted">Improves title-search stability</span>
        </div>

        <input
          v-model="apiKeyDraft"
          type="password"
          class="settings-input"
          placeholder="Paste your Semantic Scholar API key"
        />

        <div class="settings-help">
          Your key is stored locally on this machine and injected into the backend at launch.
          If you leave this blank, title search still works with shared anonymous limits.
        </div>

        <div class="settings-links">
          <button class="settings-link" @click="openApiKeyDocs">
            Apply for an API key
          </button>
        </div>

        <div class="settings-actions">
          <button class="secondary-btn" @click="closeApiKeyPanel">Cancel</button>
          <button class="primary-btn" @click="saveApiKey">Save</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, Ref, watch } from "vue";
import SearchBar from "./components/SearchBar.vue";
import ResultCard from "./components/ResultCard.vue";
import { formatBibtex } from "./utils/bibtex";

const API_BASE = "http://127.0.0.1:8765";
const API_KEY_DOCS_URL = "https://www.semanticscholar.org/product/api#api-key-form";

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
const apiKeyDraft: Ref<string> = ref("");

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
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
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
  apiKeyDraft.value = "";
}

async function saveApiKey() {
  try {
    const api = (window as any).electronAPI;
    if (!api?.saveSemanticScholarConfig) return;
    const result = await api.saveSemanticScholarConfig(apiKeyDraft.value);
    apiKeyConfigured.value = Boolean(result?.hasApiKey);
    closeApiKeyPanel();
  } catch (err) {
    console.error("Failed to save Semantic Scholar API key:", err);
    error.value = "Failed to save Semantic Scholar API key.";
  }
}

async function openApiKeyDocs() {
  try {
    const api = (window as any).electronAPI;
    if (api?.openExternalUrl) {
      await api.openExternalUrl(API_KEY_DOCS_URL);
      return;
    }
    window.open(API_KEY_DOCS_URL, "_blank", "noopener,noreferrer");
  } catch (err) {
    console.error("Failed to open Semantic Scholar API key docs:", err);
  }
}
</script>

<style>
/* Global styles wrapper */
#app-wrapper {
  width: 100vw;
  height: 100vh;
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.001);
}

.settings-panel {
  border-top: 1px solid rgba(59, 130, 246, 0.16);
  background:
    radial-gradient(circle at top left, rgba(59, 130, 246, 0.12), transparent 36%),
    linear-gradient(180deg, rgba(15, 23, 42, 0.92), rgba(15, 23, 42, 0.88));
  padding: 18px 24px 20px;
  color: #e2e8f0;
}

.settings-panel-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.settings-title {
  font-size: 15px;
  font-weight: 700;
  color: #f8fafc;
}

.settings-copy {
  margin-top: 6px;
  max-width: 520px;
  font-size: 12px;
  line-height: 1.55;
  color: rgba(226, 232, 240, 0.72);
}

.settings-close {
  flex-shrink: 0;
  border: none;
  border-radius: 999px;
  padding: 7px 12px;
  background: rgba(51, 65, 85, 0.92);
  color: #cbd5e1;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
}

.settings-badge-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 10px;
}

.settings-badge {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 0 10px;
  border-radius: 999px;
  background: rgba(37, 99, 235, 0.16);
  color: #bfdbfe;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.settings-badge-muted {
  background: rgba(51, 65, 85, 0.7);
  color: rgba(226, 232, 240, 0.78);
}

.settings-input {
  width: 100%;
  margin-top: 14px;
  padding: 11px 13px;
  border-radius: 10px;
  border: 1px solid rgba(59, 130, 246, 0.28);
  background: rgba(15, 23, 42, 0.74);
  color: #f8fafc;
  font-size: 14px;
  outline: none;
}

.settings-input:focus {
  border-color: rgba(96, 165, 250, 0.75);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.16);
}

.settings-help {
  margin-top: 8px;
  font-size: 12px;
  line-height: 1.5;
  color: rgba(148, 163, 184, 0.78);
}

.settings-links {
  margin-top: 10px;
}

.settings-link {
  padding: 0;
  border: none;
  background: transparent;
  color: #93c5fd;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.settings-link:hover {
  color: #bfdbfe;
  text-decoration: underline;
}

.settings-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 14px;
}

.primary-btn,
.secondary-btn {
  border: none;
  border-radius: 10px;
  padding: 10px 14px;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
}

.primary-btn {
  color: #eff6ff;
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
}

.secondary-btn {
  color: #cbd5e1;
  background: rgba(30, 41, 59, 0.96);
}
</style>
