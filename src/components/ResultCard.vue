<template>
  <section class="result-view" aria-live="polite">
    <div v-if="loading" class="center-state">
      <LoaderCircle class="state-spinner" :size="22" :stroke-width="1.7" aria-hidden="true" />
      <div class="state-copy">
        <p class="state-title">{{ loadingMessage }}</p>
        <p v-if="loadingDetail" class="state-detail">{{ loadingDetail }}</p>
      </div>
    </div>

    <div v-else-if="error" class="center-state">
      <CircleAlert class="error-icon" :size="22" :stroke-width="1.7" aria-hidden="true" />
      <div class="state-copy">
        <p class="state-title">{{ error }}</p>
        <p v-if="errorHint" class="state-detail">{{ errorHint }}</p>
        <button
          v-if="showApiKeyAction"
          class="text-action"
          type="button"
          @click="emit('configureApiKey')"
        >
          <KeyRound :size="14" :stroke-width="1.8" aria-hidden="true" />
          {{ apiKeyConfigured ? "Manage API key" : "Add API key" }}
        </button>
      </div>
    </div>

    <template v-else-if="bibtex">
      <div class="result-toolbar">
        <div class="result-meta">
          <FileText :size="15" :stroke-width="1.8" aria-hidden="true" />
          <span>BibTeX</span>
          <span class="type-badge" :data-type="inputType">{{ inputType }}</span>
        </div>
        <button class="copy-button" type="button" @click="emit('copy')">
          <Check v-if="copied" :size="15" :stroke-width="2" aria-hidden="true" />
          <Copy v-else :size="15" :stroke-width="1.8" aria-hidden="true" />
          {{ copied ? "Copied" : "Copy" }}
        </button>
      </div>
      <div class="code-scroll">
        <pre>{{ bibtex }}</pre>
      </div>
    </template>

    <div v-else class="empty-state">
      <div class="empty-mark" aria-label="Ready">
        <FileText :size="24" :stroke-width="1.4" aria-hidden="true" />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import {
  Check,
  CircleAlert,
  Copy,
  FileText,
  KeyRound,
  LoaderCircle,
} from "@lucide/vue";
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
  (event: "copy"): void;
  (event: "configureApiKey"): void;
}>();

const showApiKeyAction = computed(() => props.activeInputType === "title");
</script>

<style scoped>
.result-view {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  background: var(--app-bg);
  color: var(--text-main);
}

.center-state,
.empty-state {
  display: flex;
  min-height: 0;
  flex: 1;
  align-items: center;
  justify-content: center;
}

.center-state {
  gap: 12px;
  padding: 30px;
}

.state-copy {
  max-width: 450px;
}

.state-title {
  color: var(--text-main);
  font-size: 13px;
  font-weight: 550;
  line-height: 1.45;
}

.state-detail {
  margin-top: 4px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.5;
}

.state-spinner {
  flex: 0 0 auto;
  color: var(--accent);
  animation: spin 800ms linear infinite;
}

.error-icon {
  flex: 0 0 auto;
  color: var(--danger);
}

.text-action {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-top: 12px;
  border: 0;
  background: transparent;
  color: var(--accent);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
}

.text-action:hover {
  color: var(--accent-strong);
}

.result-toolbar {
  display: flex;
  min-height: 42px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 14px;
  border-bottom: 1px solid var(--border-soft);
  background: var(--surface-bg);
}

.result-meta {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 7px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 550;
}

.type-badge {
  padding: 2px 5px;
  border-radius: 4px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
}

.type-badge[data-type="arxiv"] {
  background: var(--danger-soft);
  color: var(--danger);
}

.type-badge[data-type="title"] {
  background: color-mix(in srgb, var(--warning) 12%, transparent);
  color: var(--warning);
}

.copy-button {
  display: inline-flex;
  height: 28px;
  align-items: center;
  gap: 6px;
  padding: 0 8px;
  border: 1px solid var(--border-soft);
  border-radius: 6px;
  background: var(--surface-raised);
  color: var(--text-muted);
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
  transition:
    border-color 140ms ease,
    background-color 140ms ease,
    color 140ms ease,
    transform 140ms ease;
}

.copy-button:hover {
  border-color: var(--border-strong);
  background: var(--surface-hover);
  color: var(--text-main);
}

.copy-button:active {
  transform: scale(0.97);
}

.code-scroll {
  min-height: 0;
  flex: 1;
  overflow: auto;
  padding: 16px;
}

pre {
  min-height: 100%;
  overflow-wrap: anywhere;
  white-space: pre-wrap;
  color: color-mix(in srgb, var(--text-main) 88%, var(--accent));
  font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
  font-size: 12px;
  line-height: 1.65;
}

.empty-mark {
  display: grid;
  width: 52px;
  height: 52px;
  place-items: center;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-bg);
  color: var(--text-subtle);
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
