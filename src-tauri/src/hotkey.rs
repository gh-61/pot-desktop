use crate::config::{get, set};
use crate::window::{input_translate, ocr_recognize, ocr_translate, selection_translate};
use crate::APP;
use log::{info, warn};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn register<F>(name: &str, handler: F, key: &str) -> Result<(), String>
where
    F: Fn() + Send + Sync + 'static,
{
    let hotkey = {
        if key.is_empty() {
            match get(name) {
                Some(v) => v.as_str().unwrap().to_string(),
                None => {
                    set(name, "");
                    String::new()
                }
            }
        } else {
            key.to_string()
        }
    };

    if !hotkey.is_empty() {
        let app_handle = APP.get().unwrap();
        match hotkey.parse::<tauri_plugin_global_shortcut::Shortcut>() {
            Ok(shortcut) => {
                match app_handle.global_shortcut().on_shortcut(
                    shortcut,
                    move |_app, _shortcut, _event| {
                        handler();
                    },
                ) {
                    Ok(()) => {
                        info!("Registered global shortcut: {} for {}", hotkey, name);
                    }
                    Err(e) => {
                        warn!("Failed to register global shortcut: {} {:?}", hotkey, e);
                        return Err(e.to_string());
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse shortcut: {} {:?}", hotkey, e);
                return Err(e.to_string());
            }
        }
    }
    Ok(())
}

// Register global shortcuts
pub fn register_shortcut(shortcut: &str) -> Result<(), String> {
    match shortcut {
        "hotkey_selection_translate" => {
            register("hotkey_selection_translate", selection_translate, "")?
        }
        "hotkey_input_translate" => register("hotkey_input_translate", input_translate, "")?,
        "hotkey_ocr_recognize" => register("hotkey_ocr_recognize", ocr_recognize, "")?,
        "hotkey_ocr_translate" => register("hotkey_ocr_translate", ocr_translate, "")?,
        "all" => {
            register("hotkey_selection_translate", selection_translate, "")?;
            register("hotkey_input_translate", input_translate, "")?;
            register("hotkey_ocr_recognize", ocr_recognize, "")?;
            register("hotkey_ocr_translate", ocr_translate, "")?;
        }
        _ => {}
    }
    Ok(())
}

#[tauri::command]
pub fn register_shortcut_by_frontend(name: &str, shortcut: &str) -> Result<(), String> {
    // Unregister existing shortcut for this name first
    if let Some(existing) = get(name) {
        let existing_str = existing.as_str().unwrap();
        if !existing_str.is_empty() {
            if let Ok(existing_shortcut) =
                existing_str.parse::<tauri_plugin_global_shortcut::Shortcut>()
            {
                let app_handle = APP.get().unwrap();
                let _ = app_handle.global_shortcut().unregister(existing_shortcut);
            }
        }
    }

    match name {
        "hotkey_selection_translate" => {
            register("hotkey_selection_translate", selection_translate, shortcut)?
        }
        "hotkey_input_translate" => register("hotkey_input_translate", input_translate, shortcut)?,
        "hotkey_ocr_recognize" => register("hotkey_ocr_recognize", ocr_recognize, shortcut)?,
        "hotkey_ocr_translate" => register("hotkey_ocr_translate", ocr_translate, shortcut)?,
        _ => {}
    }
    Ok(())
}
