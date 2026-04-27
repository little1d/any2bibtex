import { invoke } from "@tauri-apps/api/core";
import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Menu, MenuItem, PredefinedMenuItem, CheckMenuItem } from "@tauri-apps/api/menu";
import { TrayIcon, type TrayIconEvent } from "@tauri-apps/api/tray";
import { register } from "@tauri-apps/plugin-global-shortcut";

export type AppTheme = "dark" | "light";

export interface ResolveResponse {
  success: boolean;
  type: string;
  bibtex: string | null;
  error: string | null;
}

let tray: TrayIcon | null = null;

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
  await emit("theme-changed", savedTheme);
  await refreshTrayMenu(savedTheme);
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

export async function setupDesktopShell(): Promise<void> {
  await setupTray();
  await setupGlobalShortcut();
}

async function setupGlobalShortcut(): Promise<void> {
  try {
    await register("Alt+Space", (event) => {
      if (event.state === "Pressed") {
        void toggleWindow();
      }
    });
  } catch (error) {
    console.warn("Failed to register global shortcut:", error);
  }
}

async function setupTray(): Promise<void> {
  const theme = await getAppTheme();
  const menu = await buildTrayMenu(theme);

  tray = await TrayIcon.new({
    id: "main",
    tooltip: "any2bibtex",
    icon: "icons/tray-icon.png",
    iconAsTemplate: true,
    menu,
    showMenuOnLeftClick: false,
    action: (event: TrayIconEvent) => {
      if (event.type === "Click" && event.button === "Left" && event.buttonState === "Up") {
        void showWindow();
      }
    },
  });
}

async function refreshTrayMenu(theme: AppTheme): Promise<void> {
  if (!tray) return;
  await tray.setMenu(await buildTrayMenu(theme));
}

async function buildTrayMenu(theme: AppTheme): Promise<Menu> {
  const show = await MenuItem.new({
    text: "Show",
    action: () => void showWindow(),
  });
  const hide = await MenuItem.new({
    text: "Hide",
    action: () => void hideWindow(),
  });
  const dark = await CheckMenuItem.new({
    text: "Dark Mode",
    checked: theme === "dark",
    action: () => void setAppTheme("dark"),
  });
  const light = await CheckMenuItem.new({
    text: "Light Mode",
    checked: theme === "light",
    action: () => void setAppTheme("light"),
  });
  const separatorA = await PredefinedMenuItem.new({ item: "Separator" });
  const separatorB = await PredefinedMenuItem.new({ item: "Separator" });
  const quit = await MenuItem.new({
    text: "Quit",
    action: () => void invoke("quit_app"),
  });

  return Menu.new({
    items: [show, hide, separatorA, dark, light, separatorB, quit],
  });
}
