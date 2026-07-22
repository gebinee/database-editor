// 将 Tauri 命令返回的 AppError 对象转为中文提示
const MESSAGES = {
  WordAlreadyExists: "单词已存在",
  WordNotExists: "单词不存在",
  InvalidKey: "单词只能包含英文字母和连字符",
  DbError: "数据库错误",
  DbNotOpen: "数据库未打开",
  IoError: "IO错误",
  ExcelError: "Excel错误",
  SettingsError: "设置错误",
  Other: "错误",
};

export function errorMessage(e) {
  if (e == null) return "未知错误";
  if (typeof e === "string") return e;
  const kind = e.kind;
  const msg = e.message;
  if (kind === "Other" && typeof msg === "string") return msg;
  if (kind && typeof msg === "string") {
    return `${MESSAGES[kind] || kind}：${msg}`;
  }
  if (kind && MESSAGES[kind]) return MESSAGES[kind];
  if (typeof msg === "string") return msg;
  return String(e);
}
