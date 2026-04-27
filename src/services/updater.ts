import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

const completedUpdateKey = "any2bibtex.completedUpdate";

export interface CompletedUpdate {
  from: string;
  to: string;
}

export async function getCurrentAppVersion(): Promise<string> {
  return getVersion();
}

export async function checkForAppUpdate(): Promise<Update | null> {
  return check();
}

export async function downloadAndInstallUpdate(
  update: Update,
  onProgress: (progress: number | null) => void,
): Promise<void> {
  let downloaded = 0;
  let total: number | undefined;

  await update.downloadAndInstall((event: DownloadEvent) => {
    if (event.event === "Started") {
      downloaded = 0;
      total = event.data.contentLength;
      onProgress(total ? 0 : null);
      return;
    }

    if (event.event === "Progress") {
      downloaded += event.data.chunkLength;
      onProgress(total ? Math.round((downloaded / total) * 100) : null);
      return;
    }

    onProgress(100);
  });
}

export function markUpdateReady(from: string, to: string) {
  localStorage.setItem(completedUpdateKey, JSON.stringify({ from, to }));
}

export function hasCompletedUpdateMarker(): boolean {
  return Boolean(localStorage.getItem(completedUpdateKey));
}

export function consumeCompletedUpdate(currentVersion: string): CompletedUpdate | null {
  const rawValue = localStorage.getItem(completedUpdateKey);
  if (!rawValue) return null;

  localStorage.removeItem(completedUpdateKey);

  try {
    const parsed = JSON.parse(rawValue) as CompletedUpdate;
    return parsed.to === currentVersion ? parsed : null;
  } catch {
    return null;
  }
}

export async function relaunchApp(): Promise<void> {
  await relaunch();
}
