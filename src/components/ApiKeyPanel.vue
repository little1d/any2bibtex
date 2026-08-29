<template>
  <section class="panel-view">
    <header class="panel-header">
      <button class="icon-button" type="button" aria-label="Back" title="Back" @click="emit('close')">
        <ArrowLeft :size="18" :stroke-width="1.8" aria-hidden="true" />
      </button>
      <div class="panel-heading">
        <KeyRound :size="17" :stroke-width="1.8" aria-hidden="true" />
        <h2>Semantic Scholar</h2>
      </div>
      <span class="key-status" :data-active="apiKeyConfigured">
        <span class="status-dot"></span>
        {{ apiKeyConfigured ? "Connected" : "Optional" }}
      </span>
    </header>

    <div class="panel-body">
      <div v-if="apiKeyConfigured" class="configured-row">
        <div class="configured-icon">
          <Check :size="17" :stroke-width="2" aria-hidden="true" />
        </div>
        <div class="configured-copy">
          <strong>API key saved</strong>
          <span>Title searches use your private rate limit.</span>
        </div>
        <button
          class="danger-icon-button"
          type="button"
          aria-label="Remove API key"
          title="Remove API key"
          @click="showRevokeConfirm = true"
        >
          <Trash2 :size="16" :stroke-width="1.8" aria-hidden="true" />
        </button>
      </div>

      <form v-else class="key-form" @submit.prevent="saveApiKey">
        <label for="semantic-scholar-key">API key</label>
        <input
          id="semantic-scholar-key"
          v-model="apiKeyDraft"
          type="password"
          class="settings-input"
          placeholder="Paste key"
          autocomplete="off"
          spellcheck="false"
          autofocus
        />
        <p v-if="panelError" class="panel-error">{{ panelError }}</p>
      </form>

      <div v-if="showRevokeConfirm" class="confirm-row">
        <span>Remove the saved API key?</span>
        <div class="confirm-actions">
          <button class="button secondary" type="button" @click="showRevokeConfirm = false">
            Cancel
          </button>
          <button class="button danger" type="button" :disabled="savingApiKey" @click="removeApiKey">
            <LoaderCircle
              v-if="savingApiKey"
              class="spin"
              :size="14"
              :stroke-width="1.9"
              aria-hidden="true"
            />
            Remove
          </button>
        </div>
      </div>

      <button class="external-link" type="button" @click="openApiKeyDocs">
        Get an API key
        <ExternalLink :size="13" :stroke-width="1.8" aria-hidden="true" />
      </button>
    </div>

    <footer v-if="!apiKeyConfigured" class="panel-actions">
      <button class="button secondary" type="button" @click="emit('close')">Cancel</button>
      <button
        class="button primary"
        type="button"
        :disabled="savingApiKey || !apiKeyDraft.trim()"
        @click="saveApiKey"
      >
        <LoaderCircle
          v-if="savingApiKey"
          class="spin"
          :size="14"
          :stroke-width="1.9"
          aria-hidden="true"
        />
        Save key
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import {
  ArrowLeft,
  Check,
  ExternalLink,
  KeyRound,
  LoaderCircle,
  Trash2,
} from "@lucide/vue";
import { ref } from "vue";
import { openExternalUrl, saveSemanticScholarConfig } from "../services/desktop";

const API_KEY_DOCS_URL = "https://www.semanticscholar.org/product/api#api-key-form";

defineProps<{
  apiKeyConfigured: boolean;
}>();

const emit = defineEmits<{
  (event: "close"): void;
  (event: "saved", hasApiKey: boolean): void;
  (event: "error", message: string): void;
}>();

const apiKeyDraft = ref("");
const savingApiKey = ref(false);
const showRevokeConfirm = ref(false);
const panelError = ref("");

async function saveApiKey() {
  if (savingApiKey.value || !apiKeyDraft.value.trim()) return;
  savingApiKey.value = true;
  panelError.value = "";

  try {
    const result = await saveSemanticScholarConfig(apiKeyDraft.value);
    emit("saved", Boolean(result?.hasApiKey));
    emit("close");
  } catch (error) {
    console.error("Failed to save Semantic Scholar API key:", error);
    panelError.value = "The API key could not be saved.";
    emit("error", panelError.value);
  } finally {
    savingApiKey.value = false;
  }
}

async function removeApiKey() {
  if (savingApiKey.value) return;
  savingApiKey.value = true;
  panelError.value = "";

  try {
    const result = await saveSemanticScholarConfig("");
    emit("saved", Boolean(result?.hasApiKey));
    showRevokeConfirm.value = false;
    emit("close");
  } catch (error) {
    console.error("Failed to remove Semantic Scholar API key:", error);
    panelError.value = "The API key could not be removed.";
    emit("error", panelError.value);
  } finally {
    savingApiKey.value = false;
  }
}

async function openApiKeyDocs() {
  try {
    await openExternalUrl(API_KEY_DOCS_URL);
  } catch (error) {
    console.error("Failed to open Semantic Scholar API key docs:", error);
    panelError.value = "The link could not be opened.";
  }
}
</script>

<style scoped>
.panel-view {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  color: var(--text-main);
}

.panel-header {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr) auto;
  min-height: 52px;
  align-items: center;
  gap: 10px;
  padding: 0 14px;
  border-bottom: 1px solid var(--border-soft);
  background: var(--surface-bg);
}

.panel-heading {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 8px;
}

h2 {
  overflow: hidden;
  font-size: 13px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.icon-button,
.danger-icon-button {
  display: grid;
  width: 30px;
  height: 30px;
  place-items: center;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.icon-button:hover {
  background: var(--surface-raised);
  color: var(--text-main);
}

.key-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 600;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-subtle);
}

.key-status[data-active="true"] {
  color: var(--success);
}

.key-status[data-active="true"] .status-dot {
  background: var(--success);
}

.panel-body {
  min-height: 0;
  flex: 1;
  padding: 24px;
}

.configured-row {
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) 32px;
  align-items: center;
  gap: 12px;
  padding: 14px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-bg);
}

.configured-icon {
  display: grid;
  width: 34px;
  height: 34px;
  place-items: center;
  border-radius: 7px;
  background: color-mix(in srgb, var(--success) 12%, transparent);
  color: var(--success);
}

.configured-copy {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.configured-copy strong {
  font-size: 12px;
  font-weight: 650;
}

.configured-copy span {
  color: var(--text-muted);
  font-size: 11px;
}

.danger-icon-button:hover {
  background: var(--danger-soft);
  color: var(--danger);
}

.key-form {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 600;
}

.settings-input {
  width: 100%;
  height: 40px;
  padding: 0 11px;
  border: 1px solid var(--border-strong);
  border-radius: 7px;
  outline: 0;
  background: var(--surface-bg);
  color: var(--text-main);
  font-size: 13px;
}

.settings-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-soft);
}

.panel-error {
  color: var(--danger);
  font-size: 11px;
}

.confirm-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-top: 12px;
  padding: 10px 12px;
  border: 1px solid color-mix(in srgb, var(--danger) 24%, var(--border-soft));
  border-radius: 7px;
  background: var(--danger-soft);
  color: var(--text-main);
  font-size: 12px;
}

.confirm-actions,
.panel-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.external-link {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  margin-top: 14px;
  border: 0;
  background: transparent;
  color: var(--accent);
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
}

.external-link:hover {
  color: var(--accent-strong);
}

.panel-actions {
  min-height: 52px;
  padding: 0 14px;
  border-top: 1px solid var(--border-soft);
  background: var(--surface-bg);
}

.button {
  display: inline-flex;
  min-height: 30px;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 10px;
  border: 1px solid transparent;
  border-radius: 6px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 650;
  transition:
    background-color 140ms ease,
    border-color 140ms ease,
    color 140ms ease,
    transform 140ms ease;
}

.button.primary {
  background: var(--accent-strong);
  color: #ffffff;
}

.button.secondary {
  border-color: var(--border-soft);
  background: var(--surface-raised);
  color: var(--text-muted);
}

.button.danger {
  background: var(--danger);
  color: #ffffff;
}

.button:hover:not(:disabled) {
  filter: brightness(1.05);
}

.button:active:not(:disabled) {
  transform: scale(0.97);
}

.button:disabled {
  cursor: default;
  opacity: 0.45;
}

.spin {
  animation: spin 800ms linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
