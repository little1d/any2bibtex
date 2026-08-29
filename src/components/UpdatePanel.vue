<template>
  <section class="panel-view">
    <header class="panel-header">
      <button
        class="icon-button"
        type="button"
        :disabled="status === 'downloading'"
        aria-label="Back"
        title="Back"
        @click="close"
      >
        <ArrowLeft :size="18" :stroke-width="1.8" aria-hidden="true" />
      </button>
      <div class="panel-heading">
        <RefreshCw
          :class="{ spin: status === 'checking' }"
          :size="17"
          :stroke-width="1.8"
          aria-hidden="true"
        />
        <h2>Software update</h2>
      </div>
      <span v-if="currentVersion" class="current-version">v{{ currentVersion }}</span>
    </header>

    <div
      class="panel-body"
      :class="{ centered: !notes && status !== 'downloading' }"
    >
      <div class="status-icon" :data-status="status">
        <LoaderCircle
          v-if="status === 'checking' || status === 'downloading'"
          class="spin"
          :size="24"
          :stroke-width="1.7"
          aria-hidden="true"
        />
        <Download
          v-else-if="status === 'available'"
          :size="24"
          :stroke-width="1.7"
          aria-hidden="true"
        />
        <RotateCcw
          v-else-if="status === 'ready'"
          :size="24"
          :stroke-width="1.7"
          aria-hidden="true"
        />
        <CircleAlert
          v-else-if="status === 'error'"
          :size="24"
          :stroke-width="1.7"
          aria-hidden="true"
        />
        <CircleCheck v-else :size="24" :stroke-width="1.7" aria-hidden="true" />
      </div>

      <p v-if="targetVersion && status !== 'current'" class="target-version">
        v{{ targetVersion }}
      </p>
      <h3>{{ title }}</h3>
      <p class="summary">{{ summary }}</p>

      <div v-if="status === 'downloading'" class="progress-block">
        <div
          class="progress-track"
          role="progressbar"
          aria-label="Update download"
          :aria-valuenow="progress.percent ?? undefined"
          aria-valuemin="0"
          aria-valuemax="100"
        >
          <div
            class="progress-bar"
            :class="{ indeterminate: progress.percent === null }"
            :style="{ width: progressWidth }"
          ></div>
        </div>
        <div class="progress-meta">
          <span>{{ progressLabel }}</span>
          <span v-if="progress.percent !== null">{{ progress.percent }}%</span>
        </div>
      </div>

      <div v-if="notes && ['available', 'ready'].includes(status)" class="notes">
        <span class="notes-label">Release notes</span>
        <pre>{{ notes }}</pre>
      </div>
    </div>

    <footer class="panel-actions">
      <button
        v-if="status !== 'downloading'"
        class="button secondary"
        type="button"
        @click="close"
      >
        {{ closeLabel }}
      </button>
      <button
        v-if="status === 'available'"
        class="button primary"
        type="button"
        @click="installUpdate"
      >
        <Download :size="14" :stroke-width="1.9" aria-hidden="true" />
        Install update
      </button>
      <button
        v-else-if="status === 'ready'"
        class="button primary"
        type="button"
        @click="restart"
      >
        <RotateCcw :size="14" :stroke-width="1.9" aria-hidden="true" />
        Restart
      </button>
      <button
        v-else-if="status === 'error'"
        class="button primary"
        type="button"
        @click="checkUpdate"
      >
        <RefreshCw :size="14" :stroke-width="1.9" aria-hidden="true" />
        Try again
      </button>
      <button v-else-if="status === 'downloading'" class="button secondary" type="button" disabled>
        <LoaderCircle class="spin" :size="14" :stroke-width="1.9" aria-hidden="true" />
        Installing
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import {
  ArrowLeft,
  CircleAlert,
  CircleCheck,
  Download,
  LoaderCircle,
  RefreshCw,
  RotateCcw,
} from "@lucide/vue";
import type { Update } from "@tauri-apps/plugin-updater";
import { computed, onMounted, ref, shallowRef } from "vue";
import {
  checkForAppUpdate,
  consumeCompletedUpdate,
  downloadAndInstallUpdate,
  getCurrentAppVersion,
  markUpdateReady,
  relaunchApp,
  type CompletedUpdate,
  type UpdateProgress,
} from "../services/updater";

type UpdateStatus =
  | "checking"
  | "available"
  | "downloading"
  | "ready"
  | "current"
  | "error"
  | "success";

const props = defineProps<{
  initialUpdate?: Update | null;
}>();

const emit = defineEmits<{
  (event: "close"): void;
  (event: "updateFound", update: Update | null): void;
  (event: "busyChange", busy: boolean): void;
}>();

const status = ref<UpdateStatus>("checking");
const currentVersion = ref("");
const targetVersion = ref("");
const notes = ref("");
const errorMessage = ref("");
const progress = ref<UpdateProgress>({
  downloaded: 0,
  total: null,
  percent: null,
});
const updateRef = shallowRef<Update | null>(null);
const completedUpdate = ref<CompletedUpdate | null>(null);

const title = computed(() => {
  if (status.value === "checking") return "Checking for updates";
  if (status.value === "available") return "Update available";
  if (status.value === "downloading") return "Downloading update";
  if (status.value === "ready") return "Ready to restart";
  if (status.value === "current") return "You're up to date";
  if (status.value === "success") return "Update complete";
  return "Update check failed";
});

const summary = computed(() => {
  if (status.value === "checking") return "Looking for the latest release.";
  if (status.value === "available") return "A new version is ready to install.";
  if (status.value === "downloading") return "Keep the app open while the update is installed.";
  if (status.value === "ready") return "Restart the app to finish the update.";
  if (status.value === "current") return `Version ${currentVersion.value} is the latest release.`;
  if (status.value === "success") {
    return `Updated from ${completedUpdate.value?.from} to ${completedUpdate.value?.to}.`;
  }
  return errorMessage.value || "The update service could not be reached.";
});

const closeLabel = computed(() => {
  return status.value === "available" || status.value === "ready" ? "Later" : "Done";
});

const progressWidth = computed(() => {
  return progress.value.percent === null ? "36%" : `${progress.value.percent}%`;
});

const progressLabel = computed(() => {
  const downloaded = formatBytes(progress.value.downloaded);
  if (!progress.value.total) return downloaded ? `${downloaded} downloaded` : "Starting download";
  return `${downloaded} of ${formatBytes(progress.value.total)}`;
});

onMounted(async () => {
  try {
    currentVersion.value = await getCurrentAppVersion();
    completedUpdate.value = consumeCompletedUpdate(currentVersion.value);

    if (completedUpdate.value) {
      targetVersion.value = completedUpdate.value.to;
      status.value = "success";
      return;
    }

    if (props.initialUpdate) {
      applyUpdate(props.initialUpdate);
      return;
    }

    await checkUpdate();
  } catch (error) {
    setError(error, "Unable to read the current app version.");
  }
});

function applyUpdate(update: Update) {
  updateRef.value = update;
  targetVersion.value = update.version;
  notes.value = update.body || "";
  status.value = "available";
  emit("updateFound", update);
}

async function checkUpdate() {
  status.value = "checking";
  errorMessage.value = "";
  progress.value = { downloaded: 0, total: null, percent: null };

  try {
    if (!currentVersion.value) {
      currentVersion.value = await getCurrentAppVersion();
    }
    const update = await checkForAppUpdate();
    if (!update) {
      status.value = "current";
      targetVersion.value = currentVersion.value;
      emit("updateFound", null);
      return;
    }
    applyUpdate(update);
  } catch (error) {
    setError(error, "Unable to check for updates.");
  }
}

async function installUpdate() {
  if (!updateRef.value) return;

  status.value = "downloading";
  progress.value = { downloaded: 0, total: null, percent: null };
  emit("busyChange", true);

  try {
    await downloadAndInstallUpdate(updateRef.value, (nextProgress) => {
      progress.value = nextProgress;
    });
    markUpdateReady(currentVersion.value, targetVersion.value);
    status.value = "ready";
  } catch (error) {
    setError(error, "Unable to install the update.");
  } finally {
    emit("busyChange", false);
  }
}

async function restart() {
  emit("busyChange", true);
  try {
    await relaunchApp();
  } catch (error) {
    emit("busyChange", false);
    setError(error, "Unable to restart the app.");
  }
}

function close() {
  if (status.value !== "downloading") emit("close");
}

function setError(error: unknown, fallback: string) {
  console.error(fallback, error);
  status.value = "error";
  errorMessage.value = fallback;
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "";
  const units = ["B", "KB", "MB", "GB"];
  const unitIndex = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / 1024 ** unitIndex;
  return `${value >= 10 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`;
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

.icon-button {
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

.icon-button:hover:not(:disabled) {
  background: var(--surface-raised);
  color: var(--text-main);
}

.icon-button:disabled {
  cursor: default;
  opacity: 0.35;
}

.current-version {
  color: var(--text-subtle);
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 10px;
}

.panel-body {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  overflow: auto;
  padding: 30px 32px 24px;
}

.panel-body.centered {
  justify-content: center;
  padding-bottom: 68px;
}

.status-icon {
  display: grid;
  width: 48px;
  height: 48px;
  place-items: center;
  margin-bottom: 18px;
  border: 1px solid var(--border-soft);
  border-radius: 8px;
  background: var(--surface-bg);
  color: var(--accent);
}

.status-icon[data-status="error"] {
  background: var(--danger-soft);
  color: var(--danger);
}

.status-icon[data-status="current"],
.status-icon[data-status="success"] {
  color: var(--success);
}

.status-icon[data-status="ready"] {
  color: var(--warning);
}

.target-version {
  margin-bottom: 6px;
  color: var(--accent);
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 11px;
  font-weight: 650;
}

h3 {
  color: var(--text-main);
  font-size: 20px;
  font-weight: 650;
}

.summary {
  max-width: 470px;
  margin-top: 7px;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.55;
}

.progress-block {
  margin-top: 24px;
}

.progress-track {
  width: 100%;
  height: 5px;
  overflow: hidden;
  border-radius: 4px;
  background: var(--surface-raised);
}

.progress-bar {
  height: 100%;
  border-radius: inherit;
  background: var(--accent-strong);
  transition: width 160ms ease;
}

.progress-bar.indeterminate {
  animation: progress-slide 1.1s ease-in-out infinite alternate;
}

.progress-meta {
  display: flex;
  justify-content: space-between;
  margin-top: 8px;
  color: var(--text-subtle);
  font-family: "SFMono-Regular", Consolas, monospace;
  font-size: 10px;
}

.notes {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-soft);
}

.notes-label {
  color: var(--text-subtle);
  font-size: 10px;
  font-weight: 650;
  text-transform: uppercase;
}

.notes pre {
  max-height: 164px;
  overflow: auto;
  margin-top: 9px;
  white-space: pre-wrap;
  color: var(--text-muted);
  font-family: inherit;
  font-size: 11px;
  line-height: 1.55;
}

.panel-actions {
  display: flex;
  min-height: 52px;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
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

.button:hover:not(:disabled) {
  filter: brightness(1.05);
}

.button:active:not(:disabled) {
  transform: scale(0.97);
}

.button:disabled {
  cursor: default;
  opacity: 0.55;
}

.spin {
  animation: spin 800ms linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes progress-slide {
  from {
    transform: translateX(-80%);
  }
  to {
    transform: translateX(180%);
  }
}
</style>
