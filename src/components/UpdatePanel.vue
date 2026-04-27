<template>
  <section class="update-panel">
    <header class="update-header">
      <div class="update-heading">
        <div class="update-icon">{{ status === "success" ? "✓" : "↻" }}</div>
        <h2>{{ title }}</h2>
      </div>
      <button class="icon-button" type="button" aria-label="Close update panel" @click="close">
        ×
      </button>
    </header>

    <div class="update-body">
      <p class="version" v-if="targetVersion">v{{ targetVersion }}</p>
      <p class="summary">{{ summary }}</p>

      <div v-if="status === 'downloading'" class="progress-block">
        <div class="progress-track">
          <div class="progress-bar" :style="{ width: progressWidth }"></div>
        </div>
        <p class="progress-label">
          {{ progress === null ? "Downloading update..." : `Downloading... ${progress}%` }}
        </p>
      </div>

      <div v-if="status === 'ready'" class="ready-banner">
        v{{ targetVersion }} is ready. Restart to finish the update.
      </div>

      <div v-if="status === 'success'" class="ready-banner">
        Updated from v{{ completedUpdate?.from }} to v{{ completedUpdate?.to }}.
      </div>

      <div v-if="notes" class="notes">
        <h3>Release Notes</h3>
        <pre>{{ notes }}</pre>
      </div>
    </div>

    <footer class="update-actions">
      <button class="secondary-button" type="button" @click="close">
        {{ status === "success" ? "Done" : "Later" }}
      </button>
      <button
        v-if="status === 'available'"
        class="primary-button"
        type="button"
        @click="installUpdate"
      >
        Update Now
      </button>
      <button
        v-if="status === 'ready'"
        class="primary-button"
        type="button"
        @click="restart"
      >
        Restart Now
      </button>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import type { Update } from "@tauri-apps/plugin-updater";
import {
  checkForAppUpdate,
  consumeCompletedUpdate,
  downloadAndInstallUpdate,
  getCurrentAppVersion,
  markUpdateReady,
  relaunchApp,
  type CompletedUpdate,
} from "../services/updater";

type UpdateStatus = "checking" | "available" | "downloading" | "ready" | "current" | "error" | "success";

const emit = defineEmits<{
  close: [];
}>();

const status = ref<UpdateStatus>("checking");
const currentVersion = ref("");
const targetVersion = ref("");
const notes = ref("");
const errorMessage = ref("");
const progress = ref<number | null>(null);
const updateRef = ref<Update | null>(null);
const completedUpdate = ref<CompletedUpdate | null>(null);

const title = computed(() => {
  if (status.value === "success") return "Update Complete";
  if (status.value === "current") return "You're Up to Date";
  if (status.value === "error") return "Update Check Failed";
  return "New Version";
});

const summary = computed(() => {
  if (status.value === "checking") return "Checking for updates...";
  if (status.value === "available") {
    return `Current version v${currentVersion.value}, new version is available.`;
  }
  if (status.value === "downloading") {
    return `Current version v${currentVersion.value}, downloading v${targetVersion.value}.`;
  }
  if (status.value === "ready") {
    return `Current version v${currentVersion.value}, v${targetVersion.value} has been installed.`;
  }
  if (status.value === "current") {
    return `You are already using the latest version v${currentVersion.value}.`;
  }
  if (status.value === "success") return "The update was installed successfully.";
  return errorMessage.value || "Please try again later.";
});

const progressWidth = computed(() => {
  if (progress.value === null) return "42%";
  return `${Math.min(Math.max(progress.value, 0), 100)}%`;
});

onMounted(async () => {
  currentVersion.value = await getCurrentAppVersion();
  completedUpdate.value = consumeCompletedUpdate(currentVersion.value);

  if (completedUpdate.value) {
    targetVersion.value = completedUpdate.value.to;
    status.value = "success";
    return;
  }

  await checkUpdate();
});

async function checkUpdate() {
  status.value = "checking";
  errorMessage.value = "";

  try {
    const update = await checkForAppUpdate();
    if (!update) {
      status.value = "current";
      targetVersion.value = currentVersion.value;
      return;
    }

    updateRef.value = update;
    targetVersion.value = update.version;
    notes.value = update.body || "";
    status.value = "available";
  } catch (error) {
    status.value = "error";
    errorMessage.value =
      error instanceof Error ? error.message : "Unable to check for updates.";
  }
}

async function installUpdate() {
  if (!updateRef.value) return;

  status.value = "downloading";
  progress.value = null;

  try {
    await downloadAndInstallUpdate(updateRef.value, (nextProgress) => {
      progress.value = nextProgress;
    });
    markUpdateReady(currentVersion.value, targetVersion.value);
    status.value = "ready";
  } catch (error) {
    status.value = "error";
    errorMessage.value =
      error instanceof Error ? error.message : "Unable to install the update.";
  }
}

async function restart() {
  await relaunchApp();
}

function close() {
  emit("close");
}
</script>

<style scoped>
.update-panel {
  position: absolute;
  inset: 0;
  z-index: 10;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 16px;
  background: var(--app-bg);
  color: var(--text-main);
}

.update-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 28px 32px 22px;
  border-bottom: 1px solid var(--border-soft);
}

.update-heading {
  display: flex;
  align-items: center;
  gap: 16px;
}

.update-icon {
  display: grid;
  width: 44px;
  height: 44px;
  place-items: center;
  border-radius: 14px;
  background: linear-gradient(135deg, #2563eb, #0ea5e9 52%, #14b8a6);
  color: white;
  font-size: 24px;
  font-weight: 800;
}

h2 {
  margin: 0;
  font-size: 28px;
  font-weight: 800;
  letter-spacing: -0.03em;
}

.icon-button {
  border: 0;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 32px;
  line-height: 1;
  transition:
    color 0.16s ease,
    transform 0.16s ease;
}

.icon-button:hover {
  color: var(--text-main);
  transform: rotate(4deg) scale(1.06);
}

.icon-button:active {
  transform: scale(0.96);
}

.update-body {
  flex: 1;
  overflow: hidden;
  padding: 28px 32px;
}

.version {
  margin-bottom: 12px;
  color: var(--accent);
  font-size: 20px;
  font-weight: 800;
}

.summary {
  color: var(--text-muted);
  font-size: 17px;
  line-height: 1.6;
}

.progress-block {
  margin: 28px 0 24px;
}

.progress-track {
  width: 100%;
  height: 9px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--control-bg);
}

.progress-bar {
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, #2563eb, #14b8a6);
  transition: width 160ms ease;
}

.progress-label {
  margin-top: 16px;
  text-align: center;
  color: var(--text-subtle);
  font-weight: 700;
}

.ready-banner {
  margin: 24px 0;
  padding: 16px 18px;
  border-radius: 14px;
  background: rgba(34, 197, 94, 0.12);
  color: #22c55e;
  font-weight: 800;
}

.notes {
  margin-top: 28px;
  padding-top: 22px;
  border-top: 1px solid var(--border-soft);
}

.notes h3 {
  margin: 0 0 12px;
  font-size: 18px;
}

.notes pre {
  max-height: 210px;
  overflow: auto;
  white-space: pre-wrap;
  color: var(--text-muted);
  font-family: inherit;
  font-size: 15px;
  line-height: 1.55;
}

.update-actions {
  display: flex;
  justify-content: flex-end;
  gap: 14px;
  padding: 22px 32px;
  border-top: 1px solid var(--border-soft);
  background: var(--surface-muted-bg);
}

.primary-button,
.secondary-button {
  min-width: 112px;
  border-radius: 14px;
  padding: 12px 18px;
  cursor: pointer;
  font-size: 16px;
  font-weight: 800;
  transition:
    border-color 0.16s ease,
    box-shadow 0.16s ease,
    filter 0.16s ease,
    transform 0.16s ease;
}

.primary-button {
  border: 0;
  background: linear-gradient(135deg, #2563eb, #0891b2);
  color: white;
}

.secondary-button {
  border: 1px solid var(--border-soft);
  background: var(--surface-bg);
  color: var(--text-muted);
}

.primary-button:hover,
.secondary-button:hover {
  box-shadow: 0 12px 26px var(--accent-soft);
  transform: translateY(-1px);
}

.primary-button:hover {
  filter: brightness(1.06) saturate(1.08);
}

.secondary-button:hover {
  border-color: color-mix(in srgb, var(--accent) 38%, var(--border-soft));
  color: var(--text-main);
}

.primary-button:active,
.secondary-button:active {
  transform: translateY(0) scale(0.98);
}
</style>
