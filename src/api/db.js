import { invoke } from "@tauri-apps/api/core";

// 参数使用 camelCase，Tauri v2 自动映射到 Rust 的 snake_case 形参
export const listEntries = ({ search, page, pageSize, sortOrder }) =>
  invoke("list_entries", { search, page, pageSize, sortOrder });

export const getEntry = (key) => invoke("get_entry", { key });

export const addEntry = (key, value) => invoke("add_entry", { key, value });

export const updateEntry = (key, value) => invoke("update_entry", { key, value });

export const updateEntryKey = (oldKey, newKey, value) =>
  invoke("update_entry_key", { oldKey, newKey, value });

export const deleteEntry = (key) => invoke("delete_entry", { key });

export const readExcelForImport = (path) => invoke("read_excel_for_import", { path });

export const checkDuplicates = (keys) => invoke("check_duplicates", { keys });

export const importEntries = (items) => invoke("import_entries", { items });

export const exportProblemWords = (items, path) =>
  invoke("export_problem_words", { items, path });

export const getStats = () => invoke("get_stats");
