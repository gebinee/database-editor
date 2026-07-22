use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::error::{AppError, AppResult};
use crate::excel;
use crate::models::*;
use crate::{db, settings};

/// 全局应用状态
pub struct AppState {
    /// 数据库连接（路径错误时为 None）
    pub db: Mutex<Option<Connection>>,
    /// 数据库打开失败的错误信息
    pub db_error: Mutex<Option<String>>,
    /// 数据库文件不存在（非首次安装时）需用户选择处理方式
    pub db_missing: Mutex<bool>,
    /// 应用设置
    pub settings: Mutex<Settings>,
    /// 应用数据目录
    pub app_data_dir: PathBuf,
    /// 待导入文件路径（导入窗口打开后读取并清除）
    pub pending_import_path: Mutex<Option<String>>,
}

/// 启动时返回的初始化信息
#[derive(Serialize)]
pub struct InitInfo {
    pub settings: Settings,
    pub db_error: Option<String>,
    pub db_missing: bool,
}

/// 在数据库连接上执行闭包
/// 若连接为 None 且 db_missing 为 true，尝试自动重连（用户可能已手动把文件放回原路径）
fn with_conn<F, R>(state: &AppState, f: F) -> AppResult<R>
where
    F: FnOnce(&Connection) -> AppResult<R>,
{
    let mut guard = state
        .db
        .lock()
        .map_err(|_| AppError::Other("数据库锁损坏".to_string()))?;

    // 连接为 None 时，若处于 db_missing 状态，尝试重新打开连接
    if guard.is_none() {
        let should_try = {
            let missing = state
                .db_missing
                .lock()
                .map_err(|_| AppError::Other("状态锁损坏".to_string()))?;
            *missing
        };
        if should_try {
            let path = {
                let s = state
                    .settings
                    .lock()
                    .map_err(|_| AppError::Other("设置锁损坏".to_string()))?;
                s.db_path.clone()
            };
            // 文件已存在则重新打开连接
            if std::path::Path::new(&path).exists() {
                match db::open_connection(&path) {
                    Ok(c) => {
                        *guard = Some(c);
                        // 清除 db_missing 标志
                        let mut missing = state
                            .db_missing
                            .lock()
                            .map_err(|_| AppError::Other("状态锁损坏".to_string()))?;
                        *missing = false;
                    }
                    Err(e) => {
                        return Err(AppError::DbNotOpen(format!(
                            "数据库重新连接失败：{}",
                            e.message()
                        )));
                    }
                }
            }
        }
    }

    let conn = guard
        .as_ref()
        .ok_or_else(|| AppError::DbNotOpen("数据库未打开，请在设置中配置有效的数据库路径".to_string()))?;
    f(conn)
}

/// 启动初始化信息
#[tauri::command]
pub fn init_app(state: tauri::State<'_, AppState>) -> AppResult<InitInfo> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| AppError::Other("设置锁损坏".to_string()))?
        .clone();
    let db_error = state
        .db_error
        .lock()
        .map_err(|_| AppError::Other("状态锁损坏".to_string()))?
        .clone();
    let db_missing = state
        .db_missing
        .lock()
        .map_err(|_| AppError::Other("状态锁损坏".to_string()))?
        .clone();
    Ok(InitInfo {
        settings,
        db_error,
        db_missing,
    })
}

/// 读取设置
#[tauri::command]
pub fn get_settings(state: tauri::State<'_, AppState>) -> AppResult<Settings> {
    Ok(state
        .settings
        .lock()
        .map_err(|_| AppError::Other("设置锁损坏".to_string()))?
        .clone())
}

/// 保存设置（同时持久化到磁盘，并通知导入窗口同步更新）
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> AppResult<()> {
    settings::save_settings(&state.app_data_dir, &settings)?;
    {
        let mut s = state
            .settings
            .lock()
            .map_err(|_| AppError::Other("设置锁损坏".to_string()))?;
        *s = settings;
    }
    // 通知导入窗口设置已变更，需重新应用主题/字体
    use tauri::Emitter;
    let _ = app.emit("settings:changed", ());
    Ok(())
}

/// 选择数据库文件（打开对话框）
#[tauri::command]
pub fn pick_database_file(app: AppHandle) -> AppResult<Option<String>> {
    let path = app
        .dialog()
        .file()
        .add_filter("SQLite 数据库", &["sqlite", "db", "sqlite3"])
        .blocking_pick_file();
    Ok(path
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string()))
}

/// 选择保存路径（导出用）
#[tauri::command]
pub fn pick_save_path(app: AppHandle, default_name: String) -> AppResult<Option<String>> {
    let path = app
        .dialog()
        .file()
        .set_file_name(&default_name)
        .add_filter("Excel 文件", &["xlsx"])
        .blocking_save_file();
    Ok(path
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string()))
}

/// 选择字体文件并复制到 app_data/fonts，登记到设置
#[tauri::command]
pub fn pick_font_file(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> AppResult<CustomFont> {
    let path = app
        .dialog()
        .file()
        .add_filter("字体文件", &["ttf", "otf", "woff", "woff2"])
        .blocking_pick_file();
    let picked = path
        .and_then(|p| p.into_path().ok())
        .ok_or_else(|| AppError::Other("未选择字体文件".to_string()))?;
    let src = picked.to_string_lossy().to_string();
    let font = settings::add_custom_font(&state.app_data_dir, &src)?;
    // 登记到设置并保存
    let mut s = state
        .settings
        .lock()
        .map_err(|_| AppError::Other("设置锁损坏".to_string()))?;
    // 避免重复登记同名字体
    if !s.custom_fonts.iter().any(|f| f.name == font.name) {
        s.custom_fonts.push(font.clone());
    }
    let cloned = s.clone();
    drop(s);
    settings::save_settings(&state.app_data_dir, &cloned)?;
    Ok(font)
}

/// 读取字体文件并返回 data URL
#[tauri::command]
pub fn get_font_data_url(file_path: String) -> AppResult<String> {
    settings::get_font_data_url(&file_path)
}

/// 切换数据库（打开/创建新库，更新设置）
#[tauri::command]
pub fn switch_database(
    state: tauri::State<'_, AppState>,
    new_path: String,
) -> AppResult<()> {
    match db::open_connection(&new_path) {
        Ok(conn) => {
            {
                let mut guard = state
                    .db
                    .lock()
                    .map_err(|_| AppError::Other("数据库锁损坏".to_string()))?;
                *guard = Some(conn);
            }
            {
                let mut err = state
                    .db_error
                    .lock()
                    .map_err(|_| AppError::Other("状态锁损坏".to_string()))?;
                *err = None;
            }
            {
                let mut missing = state
                    .db_missing
                    .lock()
                    .map_err(|_| AppError::Other("状态锁损坏".to_string()))?;
                *missing = false;
            }
            let mut s = state
                .settings
                .lock()
                .map_err(|_| AppError::Other("设置锁损坏".to_string()))?;
            s.db_path = new_path;
            let cloned = s.clone();
            drop(s);
            settings::save_settings(&state.app_data_dir, &cloned)?;
            Ok(())
        }
        Err(e) => {
            let msg = e.message();
            let mut err = state
                .db_error
                .lock()
                .map_err(|_| AppError::Other("状态锁损坏".to_string()))?;
            *err = Some(msg);
            Err(e)
        }
    }
}

/// 分页列表
#[tauri::command]
pub fn list_entries(
    state: tauri::State<'_, AppState>,
    search: Option<String>,
    page: u32,
    page_size: u32,
    sort_order: String,
) -> AppResult<ListResult> {
    with_conn(&state, |conn| {
        db::list_entries(conn, search.as_deref(), page, page_size, &sort_order)
    })
}

/// 单条查询
#[tauri::command]
pub fn get_entry(state: tauri::State<'_, AppState>, key: String) -> AppResult<Option<Entry>> {
    with_conn(&state, |conn| db::get_entry(conn, &key))
}

/// 添加条目
#[tauri::command]
pub fn add_entry(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    with_conn(&state, |conn| db::add_entry(conn, &key, &value))
}

/// 更新值（key 不变）
#[tauri::command]
pub fn update_entry(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    with_conn(&state, |conn| db::update_entry(conn, &key, &value))
}

/// 更新 key（事务内 delete + add）
#[tauri::command]
pub fn update_entry_key(
    state: tauri::State<'_, AppState>,
    old_key: String,
    new_key: String,
    value: String,
) -> AppResult<()> {
    with_conn(&state, |conn| {
        db::update_entry_key(conn, &old_key, &new_key, &value)
    })
}

/// 删除条目
#[tauri::command]
pub fn delete_entry(state: tauri::State<'_, AppState>, key: String) -> AppResult<()> {
    with_conn(&state, |conn| db::delete_entry(conn, &key))
}

/// 读取 Excel 返回原始行
#[tauri::command]
pub fn read_excel_for_import(path: String) -> AppResult<Vec<RawRow>> {
    excel::read_excel_for_import(path)
}

/// 批量查询哪些 key 已存在
#[tauri::command]
pub fn check_duplicates(
    state: tauri::State<'_, AppState>,
    keys: Vec<String>,
) -> AppResult<Vec<bool>> {
    with_conn(&state, |conn| db::check_duplicates(conn, keys))
}

/// 批量导入（事务，返回问题项）
#[tauri::command]
pub fn import_entries(
    state: tauri::State<'_, AppState>,
    items: Vec<Entry>,
) -> AppResult<ImportResult> {
    with_conn(&state, |conn| db::import_entries(conn, items))
}

/// 打开导入窗口，将文件路径存入状态供前端读取
#[tauri::command]
pub fn open_import_window(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    use tauri::Emitter;
    use tauri::Manager;

    // 将路径存入状态，待导入窗口加载后读取
    {
        let mut path = state
            .pending_import_path
            .lock()
            .map_err(|_| "状态锁损坏".to_string())?;
        *path = Some(file_path.clone());
    }

    // 窗口在 tauri.conf.json 中预配置（visible: false），永不销毁
    // 用户关闭窗口时会被拦截，改为隐藏（见 lib.rs 的 on_window_event）
    // 这里只需显示窗口并发送事件通知前端有新路径
    if let Some(win) = app.get_webview_window("import") {
        win.show().map_err(|e| e.to_string())?;
        win.set_focus().map_err(|e| e.to_string())?;
        app.emit("import:open", &file_path).map_err(|e| e.to_string())?;
    } else {
        return Err("导入窗口未找到".to_string());
    }
    Ok(())
}

/// 读取并清除待导入文件路径
#[tauri::command]
pub fn take_pending_import_path(
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    let mut path = state
        .pending_import_path
        .lock()
        .map_err(|_| "状态锁损坏".to_string())?;
    Ok(path.take())
}

/// 导出问题项为 Excel
#[tauri::command]
pub fn export_problem_words(items: Vec<ProblemEntry>, path: String) -> AppResult<()> {
    excel::export_problem_words(items, path)
}
