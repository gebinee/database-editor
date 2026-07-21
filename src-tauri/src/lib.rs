mod commands;
mod db;
mod error;
mod excel;
mod models;
mod settings;

use std::sync::Mutex;
use tauri::Manager;

use commands::AppState;
use error::AppError;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            use std::path::Path;

            // 应用数据目录
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::IoError(format!("无法获取应用数据目录: {}", e)))?;
            std::fs::create_dir_all(&app_data_dir)?;

            // 判断是否首次安装：settings.json 是否已存在
            // （load_or_init 在不存在时会创建，故需在调用前判断）
            let is_first_run = !settings::settings_file_path(&app_data_dir).exists();

            // 加载设置（不存在则生成默认）
            let settings = settings::load_or_init(&app_data_dir)?;

            // 打开/创建数据库
            // - 首次安装：创建默认 word.sqlite（空库）
            // - 非首次启动但库文件不存在：不自动创建，标记 db_missing 让前端提示用户
            // - 其他情况：正常打开
            let (conn, db_error, db_missing) = if is_first_run {
                // 首次安装：创建默认空库
                match db::open_connection(&settings.db_path) {
                    Ok(c) => (Some(c), None, false),
                    Err(e) => (None, Some(e.message()), false),
                }
            } else if !Path::new(&settings.db_path).exists() {
                // 非首次启动但库文件不存在：不自动创建，提示用户
                (None, None, true)
            } else {
                match db::open_connection(&settings.db_path) {
                    Ok(c) => (Some(c), None, false),
                    Err(e) => (None, Some(e.message()), false),
                }
            };

            app.manage(AppState {
                db: Mutex::new(conn),
                db_error: Mutex::new(db_error),
                db_missing: Mutex::new(db_missing),
                settings: Mutex::new(settings),
                app_data_dir,
            });

            // 注册更新器插件（仅桌面端）
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::init_app,
            commands::get_settings,
            commands::save_settings,
            commands::pick_database_file,
            commands::pick_save_path,
            commands::pick_font_file,
            commands::get_font_data_url,
            commands::switch_database,
            commands::list_entries,
            commands::get_entry,
            commands::add_entry,
            commands::update_entry,
            commands::update_entry_key,
            commands::delete_entry,
            commands::read_excel_for_import,
            commands::check_duplicates,
            commands::import_entries,
            commands::export_problem_words,
            commands::get_stats,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // 退出前显式关闭数据库连接
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app_handle.try_state::<AppState>() {
                    if let Ok(mut guard) = state.db.lock() {
                        // 取出并 drop Connection，触发 rusqlite 的关闭逻辑
                        *guard = None;
                    }
                }
            }
        });
}
