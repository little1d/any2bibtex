mod resolver;
mod settings;

use arboard::Clipboard;
use resolver::{identify_input, resolve};
use serde::Serialize;
use settings::{get_settings, normalize_semantic_scholar_api_key, save_settings_patch};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::utils::config::Color;
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "main-tray";
const MENU_SHOW: &str = "show";
const MENU_HIDE: &str = "hide";
const MENU_DARK: &str = "theme-dark";
const MENU_LIGHT: &str = "theme-light";
const MENU_QUIT: &str = "quit";

#[derive(Serialize)]
struct ResolveResponse {
    success: bool,
    #[serde(rename = "type")]
    input_type: String,
    bibtex: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct IdentifyResponse {
    #[serde(rename = "type")]
    input_type: String,
    normalized: String,
}

#[derive(Serialize)]
struct SemanticScholarConfig {
    #[serde(rename = "hasApiKey")]
    has_api_key: bool,
}

#[tauri::command]
async fn resolve_query(query: String) -> Result<ResolveResponse, String> {
    let settings = get_settings().map_err(|error| error.to_string())?;
    Ok(resolve(query, settings.semantic_scholar_api_key).await)
}

#[tauri::command]
fn identify_query(query: String) -> IdentifyResponse {
    let (input_type, normalized) = identify_input(&query);
    IdentifyResponse {
        input_type,
        normalized,
    }
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<bool, String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set_text(text)
        .map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
fn get_app_theme() -> Result<String, String> {
    let settings = get_settings().map_err(|error| error.to_string())?;
    Ok(settings.theme)
}

#[tauri::command]
fn set_app_theme(app: tauri::AppHandle, theme: String) -> Result<String, String> {
    let next_theme = if theme == "light" { "light" } else { "dark" }.to_string();
    save_settings_patch(None, Some(next_theme.clone())).map_err(|error| error.to_string())?;
    refresh_tray_menu(&app, &next_theme).map_err(|error| error.to_string())?;
    app.emit("theme-changed", next_theme.clone())
        .map_err(|error| error.to_string())?;
    Ok(next_theme)
}

#[tauri::command]
fn get_semantic_scholar_config() -> Result<SemanticScholarConfig, String> {
    let settings = get_settings().map_err(|error| error.to_string())?;
    Ok(SemanticScholarConfig {
        has_api_key: settings
            .semantic_scholar_api_key
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
    })
}

#[tauri::command]
fn save_semantic_scholar_config(api_key: String) -> Result<SemanticScholarConfig, String> {
    let normalized = normalize_semantic_scholar_api_key(&api_key);
    save_settings_patch(Some(normalized.clone()), None).map_err(|error| error.to_string())?;
    Ok(SemanticScholarConfig {
        has_api_key: !normalized.is_empty(),
    })
}

#[tauri::command]
fn open_external_url(url: String) -> Result<bool, String> {
    if url.trim().is_empty() {
        return Ok(false);
    }
    open::that(url).map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.hide();
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

fn save_theme_from_shell(app: &tauri::AppHandle, theme: &str) {
    let next_theme = if theme == "light" { "light" } else { "dark" }.to_string();
    if save_settings_patch(None, Some(next_theme.clone())).is_ok() {
        let _ = refresh_tray_menu(app, &next_theme);
        let _ = app.emit("theme-changed", next_theme);
    }
}

fn refresh_tray_menu(app: &tauri::AppHandle, theme: &str) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(build_tray_menu(app, theme)?))?;
    }
    Ok(())
}

fn build_tray_menu<R: tauri::Runtime, M: Manager<R>>(
    manager: &M,
    theme: &str,
) -> tauri::Result<Menu<R>> {
    let show = MenuItem::with_id(manager, MENU_SHOW, "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(manager, MENU_HIDE, "Hide", true, None::<&str>)?;
    let separator_a = PredefinedMenuItem::separator(manager)?;
    let dark = CheckMenuItem::with_id(
        manager,
        MENU_DARK,
        "Dark Mode",
        true,
        theme != "light",
        None::<&str>,
    )?;
    let light = CheckMenuItem::with_id(
        manager,
        MENU_LIGHT,
        "Light Mode",
        true,
        theme == "light",
        None::<&str>,
    )?;
    let separator_b = PredefinedMenuItem::separator(manager)?;
    let quit = MenuItem::with_id(manager, MENU_QUIT, "Quit", true, None::<&str>)?;

    Menu::with_items(
        manager,
        &[&show, &hide, &separator_a, &dark, &light, &separator_b, &quit],
    )
}

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let theme = get_settings()
        .map(|settings| settings.theme)
        .unwrap_or_else(|_| "dark".to_string());
    let menu = build_tray_menu(app, &theme)?;
    let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?;

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("any2bibtex")
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    show_main_window(tray.app_handle());
                }
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => show_main_window(app),
            MENU_HIDE => hide_main_window(app),
            MENU_DARK => save_theme_from_shell(app, "dark"),
            MENU_LIGHT => save_theme_from_shell(app, "light"),
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
                let _ = window.set_shadow(false);
                window.show()?;
                window.set_focus()?;
            }
            setup_tray(app.handle())?;
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_shortcuts(["alt+space"])?
                    .with_handler(|app, shortcut, event| {
                        if event.state == ShortcutState::Pressed
                            && shortcut.matches(Modifiers::ALT, Code::Space)
                        {
                            toggle_main_window(app);
                        }
                    })
                    .build(),
            )?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            resolve_query,
            identify_query,
            copy_to_clipboard,
            get_app_theme,
            set_app_theme,
            get_semantic_scholar_config,
            save_semantic_scholar_config,
            open_external_url,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
