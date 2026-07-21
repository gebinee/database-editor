use serde::Serialize;

/// 应用统一错误类型，序列化到前端后按 kind 显示对应中文提示
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    /// 单词已存在
    WordAlreadyExists,
    /// 单词不存在
    WordNotExists,
    /// 文本为空
    EmptyText,
    /// 文本非法
    InvalidText,
    /// 数据库错误
    DbError(String),
    /// 数据库未打开（路径错误等）
    DbNotOpen(String),
    /// IO 错误
    IoError(String),
    /// Excel 解析错误
    ExcelError(String),
    /// 设置错误
    SettingsError(String),
    /// 通用错误
    Other(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        // SQLite UNIQUE 约束冲突错误码 19 / SQLITE_CONSTRAINT
        if let rusqlite::Error::SqliteFailure(ref err, _) = e {
            if err.code == rusqlite::ErrorCode::ConstraintViolation {
                return AppError::WordAlreadyExists;
            }
        }
        AppError::DbError(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::IoError(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SettingsError(e.to_string())
    }
}

/// 命令返回类型别名
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// 返回人类可读的中文消息
    pub fn message(&self) -> String {
        match self {
            AppError::WordAlreadyExists => "单词已存在".to_string(),
            AppError::WordNotExists => "单词不存在".to_string(),
            AppError::EmptyText => "文本不能为空".to_string(),
            AppError::InvalidText => "文本含有非法字符".to_string(),
            AppError::DbError(m) => format!("数据库错误：{}", m),
            AppError::DbNotOpen(m) => format!("数据库未打开：{}", m),
            AppError::IoError(m) => format!("IO错误：{}", m),
            AppError::ExcelError(m) => format!("Excel错误：{}", m),
            AppError::SettingsError(m) => format!("设置错误：{}", m),
            AppError::Other(m) => m.clone(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for AppError {}
