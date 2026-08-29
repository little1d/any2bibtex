mod resolver;
mod settings;

use arboard::Clipboard;
use resolver::{identify_input, resolve};
use serde::Serialize;
use settings::{get_settings, normalize_semantic_scholar_api_key, save_settings_patch};
use tauri::image::Image;
use tauri::menu::{
    AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu,
};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::utils::config::Color;
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartManagerExt};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "main-tray";
const MENU_SHOW: &str = "show";
const MENU_HIDE: &str = "hide";
const MENU_SETTINGS: &str = "settings";
const MENU_CHECK_UPDATES: &str = "check-updates";
const MENU_LAUNCH_AT_LOGIN: &str = "launch-at-login";
const MENU_DARK: &str = "theme-dark";
const MENU_LIGHT: &str = "theme-light";
const MENU_GITHUB: &str = "github";
const MENU_QUIT: &str = "quit";
const APP_MENU_SETTINGS: &str = "app-settings";
const APP_MENU_CHECK_UPDATES: &str = "app-check-updates";
const APP_MENU_LAUNCH_AT_LOGIN: &str = "app-launch-at-login";
const APP_MENU_DARK: &str = "app-theme-dark";
const APP_MENU_LIGHT: &str = "app-theme-light";
const APP_MENU_GITHUB: &str = "app-github";
const EVENT_OPEN_SETTINGS: &str = "open-settings-panel";
const EVENT_OPEN_UPDATE: &str = "open-update-panel";
const GITHUB_URL: &str = "https://github.com/little1d/any2bibtex";

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
    refresh_shell_menus(&app, &next_theme).map_err(|error| error.to_string())?;
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
        let _ = refresh_shell_menus(app, &next_theme);
        let _ = app.emit("theme-changed", next_theme);
    }
}

fn show_panel(app: &tauri::AppHandle, event: &str) {
    show_main_window(app);
    let _ = app.emit(event, ());
}

fn toggle_launch_at_login(app: &tauri::AppHandle) {
    let autostart = app.autolaunch();
    let result = match autostart.is_enabled() {
        Ok(true) => autostart.disable(),
        Ok(false) => autostart.enable(),
        Err(error) => {
            eprintln!("failed to read autostart state: {error}");
            return;
        }
    };

    if let Err(error) = result {
        eprintln!("failed to change autostart state: {error}");
        return;
    }

    let theme = get_settings()
        .map(|settings| settings.theme)
        .unwrap_or_else(|_| "dark".to_string());
    let _ = refresh_shell_menus(app, &theme);
}

fn refresh_shell_menus(app: &tauri::AppHandle, theme: &str) -> tauri::Result<()> {
    let launch_at_login = app.autolaunch().is_enabled().unwrap_or(false);

    #[cfg(target_os = "macos")]
    app.set_menu(build_app_menu(app, theme, launch_at_login)?)?;

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_menu(Some(build_tray_menu(app, theme, launch_at_login)?))?;
    }
    Ok(())
}

fn about_item<R: tauri::Runtime, M: Manager<R>>(
    manager: &M,
) -> tauri::Result<PredefinedMenuItem<R>> {
    PredefinedMenuItem::about(
        manager,
        Some("About any2bibtex"),
        Some(AboutMetadata {
            name: Some("any2bibtex".to_string()),
            version: Some(manager.app_handle().package_info().version.to_string()),
            website: Some(GITHUB_URL.to_string()),
            website_label: Some("GitHub Repository".to_string()),
            icon: manager.app_handle().default_window_icon().cloned(),
            ..Default::default()
        }),
    )
}

#[cfg(target_os = "macos")]
fn build_app_menu(
    app: &tauri::AppHandle,
    theme: &str,
    launch_at_login: bool,
) -> tauri::Result<Menu<tauri::Wry>> {
    let about = about_item(app)?;
    let settings = MenuItem::with_id(
        app,
        APP_MENU_SETTINGS,
        "Settings...",
        true,
        Some("CmdOrCtrl+,"),
    )?;
    let check_updates = MenuItem::with_id(
        app,
        APP_MENU_CHECK_UPDATES,
        "Check for Updates...",
        true,
        None::<&str>,
    )?;
    let launch_at_login = CheckMenuItem::with_id(
        app,
        APP_MENU_LAUNCH_AT_LOGIN,
        "Launch at Login",
        true,
        launch_at_login,
        None::<&str>,
    )?;
    let app_separator_a = PredefinedMenuItem::separator(app)?;
    let app_separator_b = PredefinedMenuItem::separator(app)?;
    let app_separator_c = PredefinedMenuItem::separator(app)?;
    let app_separator_d = PredefinedMenuItem::separator(app)?;
    let services = PredefinedMenuItem::services(app, None)?;
    let hide = PredefinedMenuItem::hide(app, None)?;
    let hide_others = PredefinedMenuItem::hide_others(app, None)?;
    let show_all = PredefinedMenuItem::show_all(app, None)?;
    let quit = PredefinedMenuItem::quit(app, None)?;
    let app_menu = Submenu::with_items(
        app,
        "any2bibtex",
        true,
        &[
            &about,
            &app_separator_a,
            &settings,
            &check_updates,
            &app_separator_b,
            &launch_at_login,
            &app_separator_c,
            &services,
            &app_separator_d,
            &hide,
            &hide_others,
            &show_all,
            &quit,
        ],
    )?;

    let undo = PredefinedMenuItem::undo(app, None)?;
    let redo = PredefinedMenuItem::redo(app, None)?;
    let edit_separator_a = PredefinedMenuItem::separator(app)?;
    let cut = PredefinedMenuItem::cut(app, None)?;
    let copy = PredefinedMenuItem::copy(app, None)?;
    let paste = PredefinedMenuItem::paste(app, None)?;
    let select_all = PredefinedMenuItem::select_all(app, None)?;
    let edit_separator_b = PredefinedMenuItem::separator(app)?;
    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &undo,
            &redo,
            &edit_separator_a,
            &cut,
            &copy,
            &paste,
            &edit_separator_b,
            &select_all,
        ],
    )?;

    let dark = CheckMenuItem::with_id(
        app,
        APP_MENU_DARK,
        "Dark Mode",
        true,
        theme != "light",
        None::<&str>,
    )?;
    let light = CheckMenuItem::with_id(
        app,
        APP_MENU_LIGHT,
        "Light Mode",
        true,
        theme == "light",
        None::<&str>,
    )?;
    let fullscreen = PredefinedMenuItem::fullscreen(app, None)?;
    let view_separator = PredefinedMenuItem::separator(app)?;
    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[&dark, &light, &view_separator, &fullscreen],
    )?;

    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let close = PredefinedMenuItem::close_window(app, None)?;
    let window_menu = Submenu::with_items(app, "Window", true, &[&minimize, &close])?;

    let github = MenuItem::with_id(
        app,
        APP_MENU_GITHUB,
        "GitHub Repository",
        true,
        None::<&str>,
    )?;
    let help_menu = Submenu::with_items(app, "Help", true, &[&github])?;

    Menu::with_items(
        app,
        &[&app_menu, &edit_menu, &view_menu, &window_menu, &help_menu],
    )
}

fn handle_app_menu_event(app: &tauri::AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        APP_MENU_SETTINGS => show_panel(app, EVENT_OPEN_SETTINGS),
        APP_MENU_CHECK_UPDATES => show_panel(app, EVENT_OPEN_UPDATE),
        APP_MENU_LAUNCH_AT_LOGIN => toggle_launch_at_login(app),
        APP_MENU_DARK => save_theme_from_shell(app, "dark"),
        APP_MENU_LIGHT => save_theme_from_shell(app, "light"),
        APP_MENU_GITHUB => {
            let _ = open::that(GITHUB_URL);
        }
        _ => {}
    }
}

fn build_tray_menu<R: tauri::Runtime, M: Manager<R>>(
    manager: &M,
    theme: &str,
    launch_at_login: bool,
) -> tauri::Result<Menu<R>> {
    let show = MenuItem::with_id(
        manager,
        MENU_SHOW,
        "Open any2bibtex",
        true,
        Some("Alt+Space"),
    )?;
    let hide = MenuItem::with_id(manager, MENU_HIDE, "Hide Window", true, None::<&str>)?;
    let settings =
        MenuItem::with_id(manager, MENU_SETTINGS, "Settings...", true, None::<&str>)?;
    let check_updates = MenuItem::with_id(
        manager,
        MENU_CHECK_UPDATES,
        "Check for Updates...",
        true,
        None::<&str>,
    )?;
    let separator_a = PredefinedMenuItem::separator(manager)?;
    let launch_at_login = CheckMenuItem::with_id(
        manager,
        MENU_LAUNCH_AT_LOGIN,
        "Launch at Login",
        true,
        launch_at_login,
        None::<&str>,
    )?;
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
    let appearance =
        Submenu::with_id_and_items(manager, "appearance", "Appearance", true, &[&dark, &light])?;
    let separator_b = PredefinedMenuItem::separator(manager)?;
    let github = MenuItem::with_id(
        manager,
        MENU_GITHUB,
        "GitHub Repository",
        true,
        None::<&str>,
    )?;
    let about = about_item(manager)?;
    let separator_c = PredefinedMenuItem::separator(manager)?;
    let quit = MenuItem::with_id(manager, MENU_QUIT, "Quit", true, None::<&str>)?;

    Menu::with_items(
        manager,
        &[
            &show,
            &hide,
            &settings,
            &check_updates,
            &separator_a,
            &launch_at_login,
            &appearance,
            &separator_b,
            &github,
            &about,
            &separator_c,
            &quit,
        ],
    )
}

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let theme = get_settings()
        .map(|settings| settings.theme)
        .unwrap_or_else(|_| "dark".to_string());
    let launch_at_login = app.autolaunch().is_enabled().unwrap_or(false);
    let menu = build_tray_menu(app, &theme, launch_at_login)?;
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
            MENU_SETTINGS => show_panel(app, EVENT_OPEN_SETTINGS),
            MENU_CHECK_UPDATES => show_panel(app, EVENT_OPEN_UPDATE),
            MENU_LAUNCH_AT_LOGIN => toggle_launch_at_login(app),
            MENU_DARK => save_theme_from_shell(app, "dark"),
            MENU_LIGHT => save_theme_from_shell(app, "light"),
            MENU_GITHUB => {
                let _ = open::that(GITHUB_URL);
            }
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .on_menu_event(handle_app_menu_event)
        .setup(|app| {
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
                let _ = window.set_shadow(false);
                if std::env::args().any(|argument| argument == "--hidden") {
                    window.hide()?;
                } else {
                    window.show()?;
                    window.set_focus()?;
                }
            }
            #[cfg(target_os = "macos")]
            {
                let theme = get_settings()
                    .map(|settings| settings.theme)
                    .unwrap_or_else(|_| "dark".to_string());
                let launch_at_login = app.autolaunch().is_enabled().unwrap_or(false);
                app.set_menu(build_app_menu(app.handle(), &theme, launch_at_login)?)?;
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
