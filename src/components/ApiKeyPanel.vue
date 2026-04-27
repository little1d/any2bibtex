<template>
  <div class="settings-backdrop" @click.self="emit('close')">
    <div class="settings-panel">
      <div class="settings-panel-header">
        <div>
          <div class="settings-title">Semantic Scholar API Key</div>
          <div class="settings-copy">
            Title search works without a key, but anonymous requests use shared rate limits.
            Adding your own key makes title search more reliable.
          </div>
        </div>
      </div>

      <div class="settings-badge-row">
        <span class="settings-badge">{{ apiKeyConfigured ? "Configured" : "Optional" }}</span>
        <span class="settings-badge settings-badge-muted">Limit: 1 request/second</span>
      </div>

      <div v-if="apiKeyConfigured" class="configured-box">
        <div>
          <div class="configured-title">API key is active</div>
          <div class="configured-copy">
            Title search will use your local Semantic Scholar API key.
          </div>
        </div>
        <button class="danger-inline-btn" @click="showRevokeConfirm = true">
          Remove
        </button>
      </div>

      <template v-else>
        <input
          v-model="apiKeyDraft"
          type="password"
          class="settings-input"
          placeholder="Paste API key"
        />

        <div class="settings-help">
          Your key is stored locally on this machine and used by the Rust resolver.
          If you leave this blank, title search still works with shared anonymous limits.
        </div>
      </template>

      <div class="settings-links">
        <button class="settings-link" @click="openApiKeyDocs">
          Apply for an API key
        </button>
      </div>

      <div class="settings-actions">
        <button class="secondary-btn" @click="emit('close')">
          {{ apiKeyConfigured ? "Close" : "Cancel" }}
        </button>
        <button
          v-if="!apiKeyConfigured"
          class="primary-btn"
          :disabled="savingApiKey"
          @click="saveApiKey"
        >
          {{ savingApiKey ? "Saving..." : "Save" }}
        </button>
      </div>

      <div
        v-if="showRevokeConfirm"
        class="confirm-backdrop"
        @click.self="showRevokeConfirm = false"
      >
        <div class="confirm-dialog">
          <div class="confirm-title">Remove API key?</div>
          <div class="confirm-copy">
            Title search will continue with shared anonymous Semantic Scholar limits.
          </div>
          <div class="confirm-actions">
            <button class="secondary-btn" @click="showRevokeConfirm = false">Cancel</button>
            <button class="danger-btn" :disabled="savingApiKey" @click="removeApiKey">
              {{ savingApiKey ? "Removing..." : "Remove key" }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { openExternalUrl, saveSemanticScholarConfig } from "../services/desktop";

const API_KEY_DOCS_URL = "https://www.semanticscholar.org/product/api#api-key-form";

defineProps<{
  apiKeyConfigured: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "saved", hasApiKey: boolean): void;
  (e: "error", message: string): void;
}>();

const apiKeyDraft = ref("");
const savingApiKey = ref(false);
const showRevokeConfirm = ref(false);

async function saveApiKey() {
  if (savingApiKey.value) return;
  savingApiKey.value = true;

  try {
    const result = await saveSemanticScholarConfig(apiKeyDraft.value);
    emit("saved", Boolean(result?.hasApiKey));
    emit("close");
  } catch (err) {
    console.error("Failed to save Semantic Scholar API key:", err);
    emit("error", "Failed to save Semantic Scholar API key.");
  } finally {
    savingApiKey.value = false;
  }
}

async function removeApiKey() {
  if (savingApiKey.value) return;
  savingApiKey.value = true;

  try {
    const result = await saveSemanticScholarConfig("");
    emit("saved", Boolean(result?.hasApiKey));
    showRevokeConfirm.value = false;
    emit("close");
  } catch (err) {
    console.error("Failed to remove Semantic Scholar API key:", err);
    emit("error", "Failed to remove Semantic Scholar API key.");
  } finally {
    savingApiKey.value = false;
  }
}

async function openApiKeyDocs() {
  try {
    await openExternalUrl(API_KEY_DOCS_URL);
  } catch (err) {
    console.error("Failed to open Semantic Scholar API key docs:", err);
  }
}
</script>

<style scoped>
.settings-backdrop {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: transparent;
}

.settings-panel {
  position: relative;
  width: min(520px, calc(100vw - 40px));
  overflow: hidden;
  border-radius: 16px;
  border: 1px solid var(--border-soft);
  background: var(--surface-bg);
  padding: 18px 20px 20px;
  color: var(--text-main);
}

.settings-panel-header {
  display: flex;
  align-items: flex-start;
}

.settings-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-main);
}

.settings-copy {
  margin-top: 6px;
  max-width: 520px;
  font-size: 12px;
  line-height: 1.55;
  color: var(--text-muted);
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
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.01em;
}

.settings-badge-muted {
  background: var(--control-bg);
  color: var(--text-muted);
}

.configured-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  margin-top: 14px;
  padding: 14px;
  border-radius: 12px;
  border: 1px solid var(--success-border);
  background: var(--success-bg);
}

.configured-title {
  color: var(--success-text);
  font-size: 13px;
  font-weight: 700;
}

.configured-copy {
  margin-top: 4px;
  color: var(--success-muted);
  font-size: 12px;
  line-height: 1.45;
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
  color: var(--text-muted);
}

.settings-links {
  margin-top: 10px;
}

.settings-link {
  padding: 0;
  border: none;
  background: transparent;
  color: var(--accent);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.settings-link:hover {
  color: var(--accent);
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
  transition:
    box-shadow 0.16s ease,
    transform 0.16s ease,
    filter 0.16s ease,
    background-color 0.16s ease;
}

.primary-btn:disabled,
.secondary-btn:disabled {
  cursor: not-allowed;
  opacity: 0.62;
}

.primary-btn {
  color: #eff6ff;
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
}

.secondary-btn {
  color: var(--text-main);
  background: var(--control-bg);
}

.primary-btn:hover:not(:disabled),
.secondary-btn:hover:not(:disabled) {
  box-shadow: 0 10px 24px var(--accent-soft);
  transform: translateY(-1px);
}

.primary-btn:hover:not(:disabled) {
  filter: brightness(1.06) saturate(1.08);
}

.primary-btn:active:not(:disabled),
.secondary-btn:active:not(:disabled) {
  transform: translateY(0) scale(0.98);
}

.danger-inline-btn {
  flex-shrink: 0;
  border: 1px solid var(--danger-border);
  border-radius: 10px;
  padding: 8px 12px;
  color: var(--danger-text);
  background: var(--danger-soft);
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition:
    box-shadow 0.16s ease,
    transform 0.16s ease,
    filter 0.16s ease;
}

.danger-inline-btn:hover {
  box-shadow: 0 10px 24px color-mix(in srgb, var(--danger-text) 18%, transparent);
  filter: brightness(1.04);
  transform: translateY(-1px);
}

.danger-inline-btn:active {
  transform: translateY(0) scale(0.98);
}

.confirm-backdrop {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px;
  background: rgba(2, 6, 23, 0.62);
}

.confirm-dialog {
  width: min(360px, 100%);
  border-radius: 14px;
  border: 1px solid rgba(148, 163, 184, 0.18);
  background: #0f172a;
  padding: 16px;
}

.confirm-title {
  color: #f8fafc;
  font-size: 15px;
  font-weight: 700;
}

.confirm-copy {
  margin-top: 8px;
  color: rgba(203, 213, 225, 0.78);
  font-size: 12px;
  line-height: 1.5;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 16px;
}

.danger-btn {
  border: none;
  border-radius: 10px;
  padding: 10px 14px;
  color: #fee2e2;
  background: #b91c1c;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition:
    box-shadow 0.16s ease,
    transform 0.16s ease,
    filter 0.16s ease;
}

.danger-btn:hover:not(:disabled) {
  box-shadow: 0 10px 24px rgba(185, 28, 28, 0.24);
  filter: brightness(1.05);
  transform: translateY(-1px);
}

.danger-btn:active:not(:disabled) {
  transform: translateY(0) scale(0.98);
}

.danger-btn:disabled {
  cursor: not-allowed;
  opacity: 0.62;
}

:global(:root) {
  --success-bg: rgba(20, 83, 45, 0.18);
  --success-border: rgba(34, 197, 94, 0.18);
  --success-text: #bbf7d0;
  --success-muted: rgba(220, 252, 231, 0.68);
  --danger-soft: rgba(127, 29, 29, 0.32);
  --danger-border: rgba(248, 113, 113, 0.24);
  --danger-text: #fecaca;
}

:global(:root[data-theme="light"]) {
  --success-bg: #ecfdf5;
  --success-border: #86efac;
  --success-text: #047857;
  --success-muted: #166534;
  --danger-soft: #fee2e2;
  --danger-border: #fca5a5;
  --danger-text: #b91c1c;
}
</style>
