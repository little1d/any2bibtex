import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

const completedUpdateKey = "any2bibtex.completedUpdate";
const markerMaxAgeMs = 7 * 24 * 60 * 60 * 1000;

export interface CompletedUpdate {
  from: string;
  to: string;
  installedAt?: number;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
  percent: number | null;
}

export async function getCurrentAppVersion(): Promise<string> {
  return getVersion();
}

export async function checkForAppUpdate(): Promise<Update | null> {
  return check();
}

export async function downloadAndInstallUpdate(
  update: Update,
  onProgress: (progress: UpdateProgress) => void,
): Promise<void> {
  let downloaded = 0;
  let total: number | null = null;

  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === "Started") {
      downloaded = 0;
      total = event.data.contentLength ?? null;
    } else if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
    } else {
      downloaded = total ?? downloaded;
    }

    const percent =
      total && total > 0
        ? Math.min(100, Math.round((downloaded / total) * 100))
        : event.event === "Finished"
          ? 100
          : null;

    onProgress({ downloaded, total, percent });
  });
}

export function markUpdateReady(from: string, to: string) {
  const marker: CompletedUpdate = {
    from,
    to,
    installedAt: Date.now(),
  };

  try {
    localStorage.setItem(completedUpdateKey, JSON.stringify(marker));
  } catch (error) {
    console.warn("Failed to persist the completed update marker:", error);
  }
}

export function hasCompletedUpdateMarker(): boolean {
  try {
    return Boolean(localStorage.getItem(completedUpdateKey));
  } catch {
    return false;
  }
}

export function consumeCompletedUpdate(currentVersion: string): CompletedUpdate | null {
  let rawValue: string | null;

  try {
    rawValue = localStorage.getItem(completedUpdateKey);
  } catch {
    return null;
  }

  if (!rawValue) return null;

  try {
    const marker = JSON.parse(rawValue) as CompletedUpdate;
    const validMarker =
      typeof marker.from === "string" &&
      typeof marker.to === "string" &&
      marker.from.length > 0 &&
      marker.to.length > 0;

    if (!validMarker) {
      localStorage.removeItem(completedUpdateKey);
      return null;
    }

    if (marker.to === currentVersion) {
      localStorage.removeItem(completedUpdateKey);
      return marker;
    }

    const installedAt = marker.installedAt ?? 0;
    if (installedAt > 0 && Date.now() - installedAt > markerMaxAgeMs) {
      localStorage.removeItem(completedUpdateKey);
    }
  } catch {
    localStorage.removeItem(completedUpdateKey);
  }

  return null;
}

export async function relaunchApp(): Promise<void> {
  await relaunch();
}
