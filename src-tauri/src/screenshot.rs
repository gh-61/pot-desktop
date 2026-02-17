use log::{info, warn};

#[tauri::command]
pub fn screenshot(x: i32, y: i32) -> Result<(), String> {
    use crate::APP;
    use dirs::cache_dir;
    use screenshots::{Compression, Screen};
    use std::fs;
    info!("Screenshot screen with position: x={}, y={}", x, y);
    let screens = Screen::all().map_err(|e| {
        warn!("Failed to get screens: {:?}", e);
        format!("Failed to get screens: {:?}", e)
    })?;
    info!("Found {} screens", screens.len());
    for screen in &screens {
        info!("Screen: {:?}", screen.display_info);
    }
    // Try exact match first, then fall back to first screen
    let target_screen = screens.iter().find(|s| s.display_info.x == x && s.display_info.y == y);
    let screen = match target_screen {
        Some(s) => s,
        None => {
            warn!("No screen matched position ({}, {}), using first screen", x, y);
            screens.first().ok_or_else(|| "No screens found".to_string())?
        }
    };

    let handle = APP.get().unwrap();
    let mut app_cache_dir_path = cache_dir().expect("Get Cache Dir Failed");
    app_cache_dir_path.push(&handle.config().identifier);
    if !app_cache_dir_path.exists() {
        fs::create_dir_all(&app_cache_dir_path).expect("Create Cache Dir Failed");
    }
    app_cache_dir_path.push("pot_screenshot.png");

    let image = screen.capture().map_err(|e| {
        warn!("Failed to capture screen: {:?}", e);
        format!("Failed to capture screen: {:?}", e)
    })?;
    let buffer = image.to_png(Compression::Fast).map_err(|e| {
        warn!("Failed to encode screenshot: {:?}", e);
        format!("Failed to encode screenshot: {:?}", e)
    })?;
    fs::write(&app_cache_dir_path, buffer).map_err(|e| {
        warn!("Failed to write screenshot: {:?}", e);
        format!("Failed to write screenshot: {:?}", e)
    })?;
    info!("Screenshot saved to {:?}", app_cache_dir_path);
    Ok(())
}
