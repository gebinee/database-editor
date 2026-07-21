use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::models::{CustomFont, Settings};

/// 设置文件名
const SETTINGS_FILE: &str = "settings.json";
/// 字体子目录
const FONTS_DIR: &str = "fonts";

/// 获取设置文件路径
pub fn settings_file_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(SETTINGS_FILE)
}

/// 获取字体目录路径
pub fn fonts_dir_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(FONTS_DIR)
}

/// 加载设置；不存在则生成默认并写入磁盘
pub fn load_or_init(app_data_dir: &Path) -> AppResult<Settings> {
    let path = settings_file_path(app_data_dir);
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        let settings: Settings = serde_json::from_str(&content)?;
        Ok(settings)
    } else {
        // 默认数据库放在 app_data_dir 下
        let db_path = app_data_dir.join("word.sqlite");
        let settings = Settings {
            db_path: db_path.to_string_lossy().to_string(),
            ..Settings::default()
        };
        save_settings(app_data_dir, &settings)?;
        Ok(settings)
    }
}

/// 保存设置到磁盘
pub fn save_settings(app_data_dir: &Path, settings: &Settings) -> AppResult<()> {
    std::fs::create_dir_all(app_data_dir)?;
    let path = settings_file_path(app_data_dir);
    let content = serde_json::to_string_pretty(settings)?;
    std::fs::write(&path, content)?;
    // 同步写一份纯文本数据库路径，供 NSIS 卸载钩子读取（避免在 NSIS 中解析 JSON）
    let db_path_file = app_data_dir.join("db_path.txt");
    std::fs::write(&db_path_file, &settings.db_path)?;
    Ok(())
}

/// 将用户选择的字体文件复制到 app_data/fonts/，并返回登记项
pub fn add_custom_font(app_data_dir: &Path, src_path: &str) -> AppResult<CustomFont> {
    let src = Path::new(src_path);
    let filename = src
        .file_name()
        .ok_or_else(|| AppError::Other("无效的字体文件路径".to_string()))?
        .to_string_lossy()
        .to_string();

    let fonts_dir = fonts_dir_path(app_data_dir);
    std::fs::create_dir_all(&fonts_dir)?;

    let dest = fonts_dir.join(&filename);
    std::fs::copy(src, &dest)?;

    // 字体名 = 文件名去掉扩展名
    let name = src
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(filename.clone());

    Ok(CustomFont {
        name,
        file_path: dest.to_string_lossy().to_string(),
    })
}

/// 读取字体文件并返回 data URL（供前端注入 @font-face）
pub fn get_font_data_url(file_path: &str) -> AppResult<String> {
    use base64_encode::encode;
    let bytes = std::fs::read(file_path)?;
    // 根据 MIME 类型，ttf/otf/woff/woff2
    let ext = Path::new(file_path)
        .extension()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| "ttf".to_string());
    let mime = match ext.as_str() {
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "font/ttf",
    };
    Ok(format!("data:{};base64,{}", mime, encode(&bytes)))
}

/// 极简 base64 编码（避免引入额外 crate）
mod base64_encode {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(input: &[u8]) -> String {
        let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
        let mut chunks = input.chunks_exact(3);
        for c in &mut chunks {
            let n = ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32);
            out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
            out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
            out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
            out.push(TABLE[(n & 0x3F) as usize] as char);
        }
        let rem = chunks.remainder();
        match rem.len() {
            1 => {
                let n = (rem[0] as u32) << 16;
                out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
                out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
                out.push('=');
                out.push('=');
            }
            2 => {
                let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
                out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
                out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
                out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
                out.push('=');
            }
            _ => {}
        }
        out
    }
}
