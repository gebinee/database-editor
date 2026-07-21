use rusqlite::{Connection, OpenFlags};

use crate::error::{AppError, AppResult};
use crate::models::{Entry, ImportResult, ListResult, ProblemEntry, Stats};

/// 打开/创建数据库连接并初始化 schema
pub fn open_connection(path: &str) -> AppResult<Connection> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_URI,
    )?;
    // 使用默认的 DELETE 模式
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS kv_store (key TEXT PRIMARY KEY, value TEXT NOT NULL);")?;
    Ok(conn)
}

/// 校验排序方向，防 SQL 注入
fn validate_sort_order(order: &str) -> &'static str {
    if order.eq_ignore_ascii_case("desc") {
        "DESC"
    } else {
        "ASC"
    }
}

/// 列出条目（分页 + 搜索 + 排序）
pub fn list_entries(
    conn: &Connection,
    search: Option<&str>,
    page: u32,
    page_size: u32,
    sort_order: &str,
) -> AppResult<ListResult> {
    let order = validate_sort_order(sort_order);
    let page_size = if page_size == 0 { 50 } else { page_size as i64 };
    let offset = ((page.saturating_sub(1)) as i64) * page_size;

    let (total, items) = match search {
        Some(s) if !s.trim().is_empty() => {
            let pattern = format!("%{}%", s.trim());
            let total: u64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM kv_store WHERE key LIKE ?1",
                    rusqlite::params![&pattern],
                    |row| row.get::<_, i64>(0),
                )? as u64;
            let mut stmt = conn.prepare(
                &format!("SELECT key, value FROM kv_store WHERE key LIKE ?1 ORDER BY key {} LIMIT ?2 OFFSET ?3", order),
            )?;
            let rows = stmt.query_map(rusqlite::params![&pattern, page_size, offset], |row| {
                Ok(Entry {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })?;
            let items: Vec<Entry> = rows.filter_map(|r| r.ok()).collect();
            (total, items)
        }
        _ => {
            let total: u64 = conn.query_row("SELECT COUNT(*) FROM kv_store", [], |row| row.get::<_, i64>(0))? as u64;
            let mut stmt = conn.prepare(
                &format!("SELECT key, value FROM kv_store ORDER BY key {} LIMIT ?1 OFFSET ?2", order),
            )?;
            let rows = stmt.query_map(rusqlite::params![page_size, offset], |row| {
                Ok(Entry {
                    key: row.get(0)?,
                    value: row.get(1)?,
                })
            })?;
            let items: Vec<Entry> = rows.filter_map(|r| r.ok()).collect();
            (total, items)
        }
    };

    Ok(ListResult { total, items })
}

/// 单条查询
pub fn get_entry(conn: &Connection, key: &str) -> AppResult<Option<Entry>> {
    let mut stmt = conn.prepare("SELECT key, value FROM kv_store WHERE key = ?1")?;
    let mut rows = stmt.query_map(rusqlite::params![key], |row| {
        Ok(Entry {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    })?;
    match rows.next() {
        Some(Ok(e)) => Ok(Some(e)),
        Some(Err(e)) => Err(AppError::from(e)),
        None => Ok(None),
    }
}

/// 添加条目
pub fn add_entry(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    match conn.execute(
        "INSERT INTO kv_store (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::from(e)),
    }
}

/// 更新值（key 不变）
pub fn update_entry(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    let affected = conn.execute(
        "UPDATE kv_store SET value = ?1 WHERE key = ?2",
        rusqlite::params![value, key],
    )?;
    if affected == 0 {
        Err(AppError::WordNotExists)
    } else {
        Ok(())
    }
}

/// 更新 key（事务内 delete + add）
pub fn update_entry_key(
    conn: &Connection,
    old_key: &str,
    new_key: &str,
    value: &str,
) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    // 检查旧 key 是否存在
    let exists: i64 =
        tx.query_row("SELECT COUNT(*) FROM kv_store WHERE key = ?1", rusqlite::params![old_key], |row| {
            row.get(0)
        })?;
    if exists == 0 {
        return Err(AppError::WordNotExists);
    }
    // 删除旧 key
    tx.execute("DELETE FROM kv_store WHERE key = ?1", rusqlite::params![old_key])?;
    // 插入新 key（若新 key 已存在则回滚并报错）
    match tx.execute(
        "INSERT INTO kv_store (key, value) VALUES (?1, ?2)",
        rusqlite::params![new_key, value],
    ) {
        Ok(_) => {
            tx.commit()?;
            Ok(())
        }
        Err(e) => {
            tx.rollback()?;
            Err(AppError::from(e))
        }
    }
}

/// 删除条目
pub fn delete_entry(conn: &Connection, key: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM kv_store WHERE key = ?1", rusqlite::params![key])?;
    if affected == 0 {
        Err(AppError::WordNotExists)
    } else {
        Ok(())
    }
}

/// 批量导入（事务内逐条插入，UNIQUE 冲突计入 problems 而非中断）
pub fn import_entries(conn: &Connection, items: Vec<Entry>) -> AppResult<ImportResult> {
    let mut inserted: u32 = 0;
    let mut problems: Vec<ProblemEntry> = Vec::new();
    let tx = conn.unchecked_transaction()?;
    for item in items {
        match tx.execute(
            "INSERT INTO kv_store (key, value) VALUES (?1, ?2)",
            rusqlite::params![item.key, item.value],
        ) {
            Ok(_) => inserted += 1,
            Err(_) => {
                // 判断是约束冲突还是其它错误
                problems.push(ProblemEntry {
                    key: item.key,
                    value: item.value,
                    problem: "单词已存在".to_string(),
                });
            }
        }
    }
    tx.commit()?;
    Ok(ImportResult { inserted, problems })
}

/// 批量查询哪些 key 已存在（供预览阶段提前标注重复）
pub fn check_duplicates(conn: &Connection, keys: Vec<String>) -> AppResult<Vec<bool>> {
    let mut stmt = conn.prepare("SELECT 1 FROM kv_store WHERE key = ?1")?;
    let mut result = Vec::with_capacity(keys.len());
    for key in &keys {
        let exists: bool = stmt
            .exists(rusqlite::params![key])
            .unwrap_or(false);
        result.push(exists);
    }
    Ok(result)
}

/// 统计信息
pub fn get_stats(conn: &Connection) -> AppResult<Stats> {
    let total_count: u64 = conn.query_row("SELECT COUNT(*) FROM kv_store", [], |row| row.get::<_, i64>(0))? as u64;
    Ok(Stats { total_count })
}
