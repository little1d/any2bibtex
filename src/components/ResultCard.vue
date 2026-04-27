<template>
  <div class="flex flex-col">
    <div v-if="loading" class="result-area border-t px-6 py-4">
      <div class="flex items-center justify-center gap-3 p-6 loading-state">
        <div class="loading-spinner"></div>
        <div class="flex flex-col gap-1.5">
          <span>{{ loadingMessage }}</span>
          <span
            v-if="loadingDetail"
            class="max-w-[420px] text-xs leading-[1.4] loading-detail"
          >
            {{ loadingDetail }}
          </span>
        </div>
      </div>
    </div>

    <div v-else-if="error" class="result-area border-t px-6 py-4">
      <div class="error-message px-4 pb-1.5 pt-2 text-center text-sm leading-[1.45]">
        {{ error }}
      </div>
      <div
        v-if="errorHint"
        class="error-hint px-5 pb-2.5 text-center text-xs leading-normal"
      >
        {{ errorHint }}
      </div>
      <div
        v-if="showApiKeyAction"
        class="action-link mx-auto"
        @click="emit('configureApiKey')"
      >
        {{ apiKeyConfigured ? "Manage Semantic Scholar API key" : "Set Semantic Scholar API key" }}
      </div>
    </div>

    <div v-else-if="bibtex" class="result-area border-t px-6 py-4">
      <pre
        class="whitespace-pre-wrap break-all rounded-lg border border-blue-500/15 bg-slate-950/80 p-4 font-mono text-[13px] leading-[1.6] text-blue-300"
      >{{ bibtex }}</pre>
    </div>

    <div
      v-else
      class="result-area empty-state flex min-h-[180px] flex-col items-center justify-center gap-2.5 border-t px-7 pb-6 pt-7"
    >
      <div class="empty-shortcut max-w-[460px] text-center text-sm font-semibold leading-[1.45]">
        Press Enter to resolve · Press Esc to hide
      </div>
      <div
        v-if="!apiKeyConfigured"
        class="empty-detail max-w-[500px] text-center text-xs leading-[1.45]"
      >
        Title search works without an API key, but uses shared Semantic Scholar rate limits.
      </div>
      <div
        class="action-link"
        @click="emit('configureApiKey')"
      >
        {{ apiKeyConfigured ? "Manage Semantic Scholar API key" : "Set Semantic Scholar API key" }}
      </div>
    </div>

    <!-- Status Bar -->
    <div
      v-if="bibtex"
      class="status-bar flex items-center justify-between border-t px-6 py-3 text-xs"
    >
      <div class="flex items-center gap-2">
        <span>Type:</span>
        <span :class="typeBadgeClass">{{ inputType }}</span>
      </div>
      <button :class="copyButtonClass" @click="handleCopy">
        {{ copied ? "Copied!" : "Copy BibTeX" }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  loading: boolean;
  loadingMessage: string;
  loadingDetail: string;
  error: string;
  errorHint: string;
  bibtex: string;
  inputType: string;
  copied: boolean;
  apiKeyConfigured: boolean;
  activeInputType: string;
}>();

const emit = defineEmits<{
  (e: "copy"): void;
  (e: "configureApiKey"): void;
}>();

function handleCopy() {
  emit("copy");
}

const showApiKeyAction = computed(() => props.activeInputType === "title");

const typeBadgeClass = computed(() => [
  "rounded px-2 py-0.5 text-[11px] font-semibold uppercase text-white",
  props.inputType === "doi" && "bg-blue-500",
  props.inputType === "arxiv" && "bg-red-500",
  props.inputType === "title" && "bg-violet-500",
]);

const copyButtonClass = computed(() => [
  "rounded-md px-4 py-1.5 text-xs font-semibold text-white transition-all hover:-translate-y-px hover:shadow-[0_4px_12px_rgba(59,130,246,0.5)] active:translate-y-0",
  props.copied
    ? "bg-gradient-to-br from-green-500 to-green-600"
    : "bg-gradient-to-br from-blue-500 to-blue-700",
]);
</script>

<style scoped>
.result-area {
  max-height: 350px;
  overflow-y: auto;
  border-color: var(--border-soft) !important;
  background: var(--surface-muted-bg);
  color: var(--text-main);
}

.result-area,
.result-area * {
  transition: background-color 0.16s ease, color 0.16s ease, border-color 0.16s ease;
}

.result-area pre {
  border-color: var(--border-soft) !important;
  background: var(--code-bg) !important;
  color: var(--accent) !important;
}

.status-bar {
  border-color: var(--border-soft) !important;
  background: var(--surface-muted-bg);
  color: var(--text-subtle);
}

.empty-state {
  background:
    radial-gradient(circle at top, var(--accent-soft), transparent 52%),
    var(--surface-muted-bg);
}

.empty-shortcut {
  color: var(--text-muted);
}

.empty-detail,
.loading-detail {
  color: var(--text-subtle);
}

.loading-state {
  color: var(--text-muted);
}

.error-message {
  color: #ef4444;
}

.error-hint {
  color: color-mix(in srgb, #ef4444 78%, var(--text-muted));
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(59, 130, 246, 0.2);
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.action-link {
  color: var(--accent);
  font-size: 12px;
  font-weight: 600;
  text-align: center;
  cursor: pointer;
  padding: 8px 14px;
  border-radius: 999px;
  background: var(--accent-soft);
  border: 1px solid var(--border-soft);
  transition:
    border-color 0.16s ease,
    box-shadow 0.16s ease,
    transform 0.16s ease,
    background-color 0.16s ease;
}

.action-link:hover {
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 48%, transparent);
  box-shadow: 0 10px 24px var(--accent-soft);
  transform: translateY(-1px);
}

.action-link:active {
  transform: translateY(0) scale(0.98);
}

.result-area::-webkit-scrollbar {
  width: 8px;
}
.result-area::-webkit-scrollbar-track {
  background: rgba(15, 23, 42, 0.3);
  border-radius: 4px;
}
.result-area::-webkit-scrollbar-thumb {
  background: rgba(59, 130, 246, 0.4);
  border-radius: 4px;
}
</style>
