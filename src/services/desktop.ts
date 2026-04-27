import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type AppTheme = "dark" | "light";

export interface ResolveResponse {
  success: boolean;
  type: string;
  bibtex: string | null;
  error: string | null;
}

export async function resolveQuery(query: string): Promise<ResolveResponse> {
  return invoke<ResolveResponse>("resolve_query", { query });
}

export async function copyToClipboard(text: string): Promise<boolean> {
  await invoke("copy_to_clipboard", { text });
  return true;
}

export async function hideWindow(): Promise<boolean> {
  await getCurrentWindow().hide();
  return true;
}

export async function showWindow(): Promise<void> {
  const window = getCurrentWindow();
  await window.show();
  await window.setFocus();
}

export async function toggleWindow(): Promise<void> {
  const window = getCurrentWindow();
  if (await window.isVisible()) {
    await window.hide();
    return;
  }
  await showWindow();
}

export async function getAppTheme(): Promise<AppTheme> {
  const theme = await invoke<AppTheme>("get_app_theme");
  return theme === "light" ? "light" : "dark";
}

export async function setAppTheme(theme: AppTheme): Promise<AppTheme> {
  const savedTheme = await invoke<AppTheme>("set_app_theme", { theme });
  return savedTheme;
}

export async function getSemanticScholarConfig(): Promise<{ hasApiKey: boolean }> {
  return invoke("get_semantic_scholar_config");
}

export async function saveSemanticScholarConfig(apiKey: string): Promise<{ hasApiKey: boolean }> {
  return invoke("save_semantic_scholar_config", { apiKey });
}

export async function openExternalUrl(url: string): Promise<boolean> {
  return invoke("open_external_url", { url });
}

export async function onThemeChanged(callback: (theme: AppTheme) => void): Promise<UnlistenFn> {
  return listen<AppTheme>("theme-changed", (event) => callback(event.payload));
}

export async function startWindowDrag(): Promise<void> {
  await getCurrentWindow().startDragging();
}
