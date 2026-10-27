mod commands;
mod error;
mod notes;

use notes::NoteStore;
use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 第7章: 起動時にデータを読み込み State に登録
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(NoteStore::load(dir.join("notes.json"))?);

            // 第10章: メニュー（macOS の標準メニューを残したまま「メモ」を追加）
            let new_note =
                MenuItem::with_id(app, "new_note", "新規メモ", true, Some("CmdOrCtrl+N"))?;
            let notes_menu = Submenu::with_items(app, "メモ", true, &[&new_note])?;
            let menu = Menu::default(app.handle())?;
            menu.append(&notes_menu)?;
            app.set_menu(menu)?;

            // 第10章: トレイアイコン
            let show = MenuItem::with_id(app, "show", "ウィンドウを表示", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .build(app)?;
            Ok(())
        })
        // アプリメニューとトレイメニュー両方のクリックがここに来る
        .on_menu_event(|app, event| match event.id().as_ref() {
            "new_note" => {
                let _ = app.emit_to("main", "menu-new-note", ()); // main ウィンドウだけに送る
            }
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_notes,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::export_note,
            commands::analyze_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
