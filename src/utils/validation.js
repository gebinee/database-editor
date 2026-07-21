// 单词 key 校验：仅允许英文字母和连字符（与原 JavaFX 项目一致）
const KEY_REGEX = /^[a-zA-Z-]+$/;

export function checkEmptyText(text) {
  return text == null || String(text).trim() === "";
}

export function checkKeyValidity(key) {
  return KEY_REGEX.test(key);
}

export function emptyMsg() {
  return "文本不能为空";
}

export function invalidKeyMsg() {
  return "单词只能包含英文字母和连字符";
}
