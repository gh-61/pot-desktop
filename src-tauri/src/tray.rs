use crate::clipboard::*;
use crate::config::{get, set};
use crate::window::config_window;
use crate::window::input_translate;
use crate::window::ocr_recognize;
use crate::window::ocr_translate;
use crate::window::updater_window;
use log::info;
use tauri::menu::{Menu, MenuEvent, MenuItemBuilder, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager};

pub fn build_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let language = match get("app_language") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("app_language", "en");
            "en".to_string()
        }
    };
    let menu = build_tray_menu(app.handle(), &language)?;

    // Set initial selected states on menu before attaching to tray
    let copy_mode = match get("translate_auto_copy") {
        Some(v) => v.as_str().unwrap().to_string(),
        None => {
            set("translate_auto_copy", "disable");
            "disable".to_string()
        }
    };
    set_copy_mode_checked_on_menu(&menu, &copy_mode);

    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    if let Some(item) = menu.get("clipboard_monitor") {
        if let Some(check) = item.as_check_menuitem() {
            let _ = check.set_checked(enable_clipboard_monitor);
        }
    }

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(|app, event| {
            tray_menu_event_handler(app, event);
        })
        .tooltip(format!("pot {}", app.package_info().version))
        .build(app)?;

    Ok(())
}

#[tauri::command]
pub fn update_tray(app_handle: tauri::AppHandle, mut language: String, mut copy_mode: String) {
    if language.is_empty() {
        language = match get("app_language") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("app_language", "en");
                "en".to_string()
            }
        };
    }
    if copy_mode.is_empty() {
        copy_mode = match get("translate_auto_copy") {
            Some(v) => v.as_str().unwrap().to_string(),
            None => {
                set("translate_auto_copy", "disable");
                "disable".to_string()
            }
        };
    }

    info!(
        "Update tray with language: {}, copy mode: {}",
        language, copy_mode
    );

    if let Ok(menu) = build_tray_menu(&app_handle, &language) {
        // Set checked states on the new menu before attaching
        let enable_clipboard_monitor = match get("clipboard_monitor") {
            Some(v) => v.as_bool().unwrap(),
            None => {
                set("clipboard_monitor", false);
                false
            }
        };
        if let Some(item) = menu.get("clipboard_monitor") {
            if let Some(check) = item.as_check_menuitem() {
                let _ = check.set_checked(enable_clipboard_monitor);
            }
        }
        set_copy_mode_checked_on_menu(&menu, &copy_mode);

        if let Some(tray) = app_handle.tray_by_id("main") {
            let _ = tray.set_menu(Some(menu));
            #[cfg(not(target_os = "linux"))]
            let _ = tray.set_tooltip(Some(&format!("pot {}", app_handle.package_info().version)));
        }
    }
}

fn set_copy_mode_checked_on_menu(menu: &Menu<tauri::Wry>, copy_mode: &str) {
    // Uncheck all copy items first, then check the active one
    for id in &[
        "copy_source",
        "copy_target",
        "copy_source_target",
        "copy_disable",
    ] {
        if let Some(item) = menu.get(*id) {
            if let Some(check) = item.as_check_menuitem() {
                let _ = check.set_checked(false);
            }
        }
    }
    let active_id = match copy_mode {
        "source" => "copy_source",
        "target" => "copy_target",
        "source_target" => "copy_source_target",
        "disable" => "copy_disable",
        _ => return,
    };
    if let Some(item) = menu.get(active_id) {
        if let Some(check) = item.as_check_menuitem() {
            let _ = check.set_checked(true);
        }
    }
}

fn tray_menu_event_handler(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "input_translate" => on_input_translate_click(),
        "copy_source" => on_auto_copy_click(app, "source"),
        "clipboard_monitor" => on_clipboard_monitor_click(app),
        "copy_target" => on_auto_copy_click(app, "target"),
        "copy_source_target" => on_auto_copy_click(app, "source_target"),
        "copy_disable" => on_auto_copy_click(app, "disable"),
        "ocr_recognize" => on_ocr_recognize_click(),
        "ocr_translate" => on_ocr_translate_click(),
        "config" => on_config_click(),
        "check_update" => on_check_update_click(),
        "view_log" => on_view_log_click(app),
        "restart" => on_restart_click(app),
        "quit" => on_quit_click(app),
        _ => {}
    }
}

fn on_input_translate_click() {
    input_translate();
}
fn on_clipboard_monitor_click(app: &AppHandle) {
    let enable_clipboard_monitor = match get("clipboard_monitor") {
        Some(v) => v.as_bool().unwrap(),
        None => {
            set("clipboard_monitor", false);
            false
        }
    };
    let current = !enable_clipboard_monitor;
    // Update Config File
    set("clipboard_monitor", current);
    // Update State and Start Monitor
    let state = app.state::<ClipboardMonitorEnableWrapper>();
    state
        .0
        .lock()
        .unwrap()
        .replace_range(.., &current.to_string());
    if current {
        start_clipboard_monitor(app.clone());
    }
    // Rebuild tray menu to reflect updated state
    update_tray(app.clone(), "".to_string(), "".to_string());
}
fn on_auto_copy_click(app: &AppHandle, mode: &str) {
    info!("Set copy mode to: {}", mode);
    set("translate_auto_copy", mode);
    app.emit("translate_auto_copy_changed", mode).unwrap();
    update_tray(app.clone(), "".to_string(), mode.to_string());
}
fn on_ocr_recognize_click() {
    ocr_recognize();
}
fn on_ocr_translate_click() {
    ocr_translate();
}

fn on_config_click() {
    config_window();
}

fn on_check_update_click() {
    updater_window();
}
fn on_view_log_click(app: &AppHandle) {
    let log_path = app.path().app_log_dir().unwrap();
    use tauri_plugin_shell::ShellExt;
    app.shell().open(log_path.to_str().unwrap(), None).unwrap();
}
fn on_restart_click(app: &AppHandle) {
    info!("============== Restart App ==============");
    app.restart();
}
fn on_quit_click(app: &AppHandle) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    app.global_shortcut().unregister_all().unwrap();
    info!("============== Quit App ==============");
    app.exit(0);
}

// ===== Menu builders per language =====

struct TrayLabels {
    input_translate: &'static str,
    clipboard_monitor: &'static str,
    auto_copy: &'static str,
    copy_source: &'static str,
    copy_target: &'static str,
    copy_source_target: &'static str,
    copy_disable: &'static str,
    ocr_recognize: &'static str,
    ocr_translate: &'static str,
    config: &'static str,
    check_update: &'static str,
    view_log: &'static str,
    restart: &'static str,
    quit: &'static str,
}

fn get_labels(lang: &str) -> TrayLabels {
    match lang {
        "zh_cn" => TrayLabels {
            input_translate: "输入翻译",
            clipboard_monitor: "监听剪切板",
            auto_copy: "自动复制",
            copy_source: "原文",
            copy_target: "译文",
            copy_source_target: "原文+译文",
            copy_disable: "关闭",
            ocr_recognize: "文字识别",
            ocr_translate: "截图翻译",
            config: "偏好设置",
            check_update: "检查更新",
            view_log: "查看日志",
            restart: "重启应用",
            quit: "退出",
        },
        "zh_tw" => TrayLabels {
            input_translate: "輸入翻譯",
            clipboard_monitor: "偵聽剪貼簿",
            auto_copy: "自動複製",
            copy_source: "原文",
            copy_target: "譯文",
            copy_source_target: "原文+譯文",
            copy_disable: "關閉",
            ocr_recognize: "文字識別",
            ocr_translate: "截圖翻譯",
            config: "偏好設定",
            check_update: "檢查更新",
            view_log: "查看日誌",
            restart: "重啓程式",
            quit: "退出",
        },
        "ja" => TrayLabels {
            input_translate: "翻訳を入力",
            clipboard_monitor: "クリップボードを監視する",
            auto_copy: "自動コピー",
            copy_source: "原文",
            copy_target: "訳文",
            copy_source_target: "原文+訳文",
            copy_disable: "閉じる",
            ocr_recognize: "テキスト認識",
            ocr_translate: "スクリーンショットの翻訳",
            config: "プリファレンス設定",
            check_update: "更新を確認する",
            view_log: "ログを見る",
            restart: "アプリの再起動",
            quit: "退出する",
        },
        "ko" => TrayLabels {
            input_translate: "입력 번역",
            clipboard_monitor: "감청 전단판",
            auto_copy: "자동 복사",
            copy_source: "원문",
            copy_target: "번역문",
            copy_source_target: "원문+번역문",
            copy_disable: "닫기",
            ocr_recognize: "문자인식",
            ocr_translate: "스크린샷 번역",
            config: "기본 설정",
            check_update: "업데이트 확인",
            view_log: "로그 보기",
            restart: "응용 프로그램 다시 시작",
            quit: "퇴출",
        },
        "fr" => TrayLabels {
            input_translate: "Traduction d'entrée",
            clipboard_monitor: "Surveiller le presse-papiers",
            auto_copy: "Copier automatiquement",
            copy_source: "Source",
            copy_target: "Cible",
            copy_source_target: "Source+Cible",
            copy_disable: "Désactiver",
            ocr_recognize: "Reconnaissance de texte",
            ocr_translate: "Traduction d'image",
            config: "Paramètres",
            check_update: "Vérifier les mises à jour",
            view_log: "Voir le journal",
            restart: "Redémarrer l'application",
            quit: "Quitter",
        },
        "de" => TrayLabels {
            input_translate: "Eingabeübersetzung",
            clipboard_monitor: "Zwischenablage überwachen",
            auto_copy: "Automatisch kopieren",
            copy_source: "Quelle",
            copy_target: "Ziel",
            copy_source_target: "Quelle+Ziel",
            copy_disable: "Deaktivieren",
            ocr_recognize: "Texterkennung",
            ocr_translate: "Bildübersetzung",
            config: "Einstellungen",
            check_update: "Auf Updates prüfen",
            view_log: "Protokoll anzeigen",
            restart: "Anwendung neu starten",
            quit: "Beenden",
        },
        "ru" => TrayLabels {
            input_translate: "Ввод перевода",
            clipboard_monitor: "Следить за буфером обмена",
            auto_copy: "Автоматическое копирование",
            copy_source: "Источник",
            copy_target: "Цель",
            copy_source_target: "Источник+Цель",
            copy_disable: "Отключить",
            ocr_recognize: "Распознавание текста",
            ocr_translate: "Перевод изображения",
            config: "Настройки",
            check_update: "Проверить обновления",
            view_log: "Просмотр журнала",
            restart: "Перезапустить приложение",
            quit: "Выход",
        },
        "fa" => TrayLabels {
            input_translate: "متن",
            clipboard_monitor: "گوش دادن به تخته برش",
            auto_copy: "کپی خودکار",
            copy_source: "منبع",
            copy_target: "هدف",
            copy_source_target: "منبع + هدف",
            copy_disable: "متن",
            ocr_recognize: "تشخیص متن",
            ocr_translate: "ترجمه عکس",
            config: "تنظیمات ترجیح",
            check_update: "بررسی بروزرسانی",
            view_log: "مشاهده گزارشات",
            restart: "راه\u{200c}اندازی مجدد برنامه",
            quit: "خروج",
        },
        "pt_br" => TrayLabels {
            input_translate: "Traduzir Entrada",
            clipboard_monitor: "Monitorando a área de transferência",
            auto_copy: "Copiar Automaticamente",
            copy_source: "Origem",
            copy_target: "Destino",
            copy_source_target: "Origem+Destino",
            copy_disable: "Desabilitar",
            ocr_recognize: "Reconhecimento de Texto",
            ocr_translate: "Tradução de Imagem",
            config: "Configurações",
            check_update: "Checar por Atualização",
            view_log: "Exibir Registro",
            restart: "Reiniciar aplicativo",
            quit: "Sair",
        },
        "uk" => TrayLabels {
            input_translate: "Введення перекладу",
            clipboard_monitor: "Стежити за буфером обміну",
            auto_copy: "Автоматичне копіювання",
            copy_source: "Джерело",
            copy_target: "Мета",
            copy_source_target: "Джерело+Мета",
            copy_disable: "Відключивши",
            ocr_recognize: "Розпізнавання тексту",
            ocr_translate: "Переклад зображення",
            config: "Настройка",
            check_update: "Перевірити оновлення",
            view_log: "Перегляд журналу",
            restart: "Перезапустити додаток",
            quit: "Вихід",
        },
        _ => TrayLabels {
            input_translate: "Input Translate",
            clipboard_monitor: "Clipboard Monitor",
            auto_copy: "Auto Copy",
            copy_source: "Source",
            copy_target: "Target",
            copy_source_target: "Source+Target",
            copy_disable: "Disable",
            ocr_recognize: "OCR Recognize",
            ocr_translate: "OCR Translate",
            config: "Config",
            check_update: "Check Update",
            view_log: "View Log",
            restart: "Restart",
            quit: "Quit",
        },
    }
}

fn build_tray_menu(app_handle: &AppHandle, lang: &str) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let labels = get_labels(lang);

    let input_translate =
        MenuItemBuilder::with_id("input_translate", labels.input_translate).build(app_handle)?;
    let clipboard_monitor =
        tauri::menu::CheckMenuItemBuilder::with_id("clipboard_monitor", labels.clipboard_monitor)
            .build(app_handle)?;
    let copy_source = tauri::menu::CheckMenuItemBuilder::with_id("copy_source", labels.copy_source)
        .build(app_handle)?;
    let copy_target = tauri::menu::CheckMenuItemBuilder::with_id("copy_target", labels.copy_target)
        .build(app_handle)?;
    let copy_source_target =
        tauri::menu::CheckMenuItemBuilder::with_id("copy_source_target", labels.copy_source_target)
            .build(app_handle)?;
    let copy_disable =
        tauri::menu::CheckMenuItemBuilder::with_id("copy_disable", labels.copy_disable)
            .build(app_handle)?;
    let ocr_recognize =
        MenuItemBuilder::with_id("ocr_recognize", labels.ocr_recognize).build(app_handle)?;
    let ocr_translate =
        MenuItemBuilder::with_id("ocr_translate", labels.ocr_translate).build(app_handle)?;
    let config = MenuItemBuilder::with_id("config", labels.config).build(app_handle)?;
    let check_update =
        MenuItemBuilder::with_id("check_update", labels.check_update).build(app_handle)?;
    let view_log = MenuItemBuilder::with_id("view_log", labels.view_log).build(app_handle)?;
    let restart = MenuItemBuilder::with_id("restart", labels.restart).build(app_handle)?;
    let quit_item = MenuItemBuilder::with_id("quit", labels.quit).build(app_handle)?;

    let auto_copy_submenu = Submenu::with_id_and_items(
        app_handle,
        "auto_copy",
        labels.auto_copy,
        true,
        &[
            &copy_source,
            &copy_target,
            &copy_source_target,
            &PredefinedMenuItem::separator(app_handle)?,
            &copy_disable,
        ],
    )?;

    let menu = Menu::with_items(
        app_handle,
        &[
            &input_translate,
            &clipboard_monitor,
            &auto_copy_submenu,
            &PredefinedMenuItem::separator(app_handle)?,
            &ocr_recognize,
            &ocr_translate,
            &PredefinedMenuItem::separator(app_handle)?,
            &config,
            &check_update,
            &view_log,
            &PredefinedMenuItem::separator(app_handle)?,
            &restart,
            &quit_item,
        ],
    )?;

    Ok(menu)
}
