use serde::{Deserialize, Serialize};

/// 键值对条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

/// 从 Excel 读取的原始行（index 为行号，便于前端定位）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRow {
    pub index: u32,
    pub key: String,
    pub value: String,
}

/// 问题条目（导入失败原因）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemEntry {
    pub key: String,
    pub value: String,
    pub problem: String,
}

/// 批量导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub inserted: u32,
    pub problems: Vec<ProblemEntry>,
}

/// 分页列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    pub total: u64,
    pub items: Vec<Entry>,
}

/// 用户上传的自定义字体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFont {
    pub name: String,
    pub file_path: String,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// 数据库文件路径
    pub db_path: String,
    /// 全局字体大小
    pub font_size: u32,
    /// 单词字体（字体名或自定义字体名）
    pub word_font: String,
    /// 注音结果字体
    pub phonetic_font: String,
    /// UI 西文字体
    pub ui_font: String,
    /// UI 中文字体（空字符串表示不设置，跟随西文字体）
    #[serde(default)]
    pub ui_font_cn: String,
    /// 主题模式: "light" | "dark" | "auto"
    #[serde(default = "default_theme")]
    pub theme: String,
    /// 用户上传的自定义字体列表
    pub custom_fonts: Vec<CustomFont>,
}

fn default_theme() -> String {
    "auto".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        let db_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("word.sqlite")
            .to_string_lossy()
            .to_string();
        Self {
            db_path,
            font_size: 14,
            word_font: "system-ui".to_string(),
            phonetic_font: "gebinee".to_string(), // 内置字体
            ui_font: "system-ui".to_string(),
            ui_font_cn: String::new(),
            theme: "auto".to_string(),
            custom_fonts: Vec::new(),
        }
    }
}
