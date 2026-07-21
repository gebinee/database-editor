use calamine::{open_workbook_auto, Data, Reader};
use rust_xlsxwriter::Workbook;

use crate::error::{AppError, AppResult};
use crate::models::{ProblemEntry, RawRow};

/// 读取 Excel 第一个工作表，前两列作为 (key, value)，返回原始行（不做校验）
pub fn read_excel_for_import(path: String) -> AppResult<Vec<RawRow>> {
    let mut workbook =
        open_workbook_auto(&path).map_err(|e| AppError::ExcelError(e.to_string()))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::ExcelError("工作簿中没有工作表".to_string()))?
        .map_err(|e| AppError::ExcelError(e.to_string()))?;

    let mut rows = Vec::new();
    for (i, row) in range.rows().enumerate() {
        let key = row.get(0).map(cell_to_string).unwrap_or_default();
        let value = row.get(1).map(cell_to_string).unwrap_or_default();
        rows.push(RawRow {
            index: i as u32,
            key,
            value,
        });
    }
    Ok(rows)
}

/// 将 calamine 单元格转为字符串
fn cell_to_string(d: &Data) -> String {
    match d {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.is_finite() && f.trunc() == *f {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        Data::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// 导出问题单词为 Excel（三列：单词/注音结果/问题原因）
pub fn export_problem_words(items: Vec<ProblemEntry>, path: String) -> AppResult<()> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "单词")
        .map_err(|e| AppError::ExcelError(e.to_string()))?;
    worksheet
        .write_string(0, 1, "注音结果")
        .map_err(|e| AppError::ExcelError(e.to_string()))?;
    worksheet
        .write_string(0, 2, "问题原因")
        .map_err(|e| AppError::ExcelError(e.to_string()))?;

    for (i, item) in items.iter().enumerate() {
        let row = (i + 1) as u32;
        worksheet
            .write_string(row, 0, &item.key)
            .map_err(|e| AppError::ExcelError(e.to_string()))?;
        worksheet
            .write_string(row, 1, &item.value)
            .map_err(|e| AppError::ExcelError(e.to_string()))?;
        worksheet
            .write_string(row, 2, &item.problem)
            .map_err(|e| AppError::ExcelError(e.to_string()))?;
    }

    workbook
        .save(&path)
        .map_err(|e| AppError::ExcelError(e.to_string()))?;
    Ok(())
}
