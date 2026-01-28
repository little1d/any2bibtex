<template>
  <div class="result-card">
    <div v-if="loading" class="result-area">
      <div class="loading">
        <div class="loading-spinner"></div>
        <span>Resolving...</span>
      </div>
    </div>

    <div v-else-if="error" class="result-area">
      <div class="error-msg">{{ error }}</div>
    </div>

    <div v-else-if="bibtex" class="result-area">
      <pre class="bibtex-code">{{ bibtex }}</pre>
    </div>

    <div v-else class="result-area empty-state">
      <div class="hint">Press Enter to resolve · Press Esc to hide</div>
    </div>

    <!-- Status Bar -->
    <div v-if="bibtex" class="status-bar">
      <div class="status-type">
        <span>Type:</span>
        <span :class="['type-badge', inputType]">{{ inputType }}</span>
      </div>
      <button class="copy-btn" :class="{ copied: copied }" @click="handleCopy">
        {{ copied ? "Copied!" : "Copy BibTeX" }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  loading: boolean;
  error: string;
  bibtex: string;
  inputType: string;
  copied: boolean;
}>();

const emit = defineEmits<{
  (e: "copy"): void;
}>();

function handleCopy() {
  emit("copy");
}
</script>

<style scoped>
.result-card {
  display: flex;
  flex-direction: column;
}

.result-area {
  border-top: 1px solid rgba(59, 130, 246, 0.2);
  padding: 16px 24px;
  max-height: 350px;
  overflow-y: auto;
}

.empty-state {
  padding: 10px;
}

.bibtex-code {
  font-family: "SF Mono", "Monaco", "Inconsolata", "Fira Code", monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #93c5fd;
  background: rgba(15, 23, 42, 0.8);
  padding: 16px;
  border-radius: 8px;
  white-space: pre-wrap;
  word-break: break-all;
  border: 1px solid rgba(59, 130, 246, 0.15);
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 24px;
  border-top: 1px solid rgba(59, 130, 246, 0.2);
  font-size: 12px;
  color: rgba(147, 197, 253, 0.6);
  background: rgba(15, 23, 42, 0.5);
}

.status-type {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.type-badge.doi {
  background: #3b82f6;
  color: #fff;
}
.type-badge.arxiv {
  background: #ef4444;
  color: #fff;
}
.type-badge.title {
  background: #8b5cf6;
  color: #fff;
}

.copy-btn {
  padding: 6px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  background: linear-gradient(135deg, #3b82f6, #1d4ed8);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.copy-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.5);
}

.copy-btn:active {
  transform: translateY(0);
}

.copy-btn.copied {
  background: linear-gradient(135deg, #22c55e, #16a34a);
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  color: rgba(147, 197, 253, 0.7);
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(59, 130, 246, 0.2);
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-right: 12px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.error-msg {
  color: #fca5a5;
  padding: 16px;
  text-align: center;
}

.hint {
  color: rgba(147, 197, 253, 0.5);
  font-size: 13px;
  text-align: center;
  padding: 10px;
}

/* Scrollbar */
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
