mod resolver;
mod settings;

use arboard::Clipboard;
use resolver::{identify_input, resolve};
use serde::Serialize;
use settings::{get_settings, normalize_semantic_scholar_api_key, save_settings_patch};
use tauri::Manager;

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
fn set_app_theme(theme: String) -> Result<String, String> {
    let next_theme = if theme == "light" { "light" } else { "dark" }.to_string();
    save_settings_patch(None, Some(next_theme.clone())).map_err(|error| error.to_string())?;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                window.show()?;
                window.set_focus()?;
            }
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
