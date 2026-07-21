import { invoke } from "@tauri-apps/api/core";
import gebineeFontUrl from "../assets/fonts/aaae.ttf";

const injectedFonts = new Set();

function injectFontFace(name, url, format = "truetype") {
  const id = `fontface-${name}`;
  if (document.getElementById(id)) return;
  const style = document.createElement("style");
  style.id = id;
  style.textContent = `@font-face { font-family: '${name}'; src: url('${url}') format('${format}'); }`;
  document.head.appendChild(style);
}

/** 注册内置 gebinee 字体 */
export function registerBuiltinFonts() {
  if (!injectedFonts.has("gebinee")) {
    injectFontFace("gebinee", gebineeFontUrl);
    injectedFonts.add("gebinee");
  }
}

/** 注册用户上传的自定义字体（通过 Rust 读取为 data URL） */
export async function registerCustomFont(font) {
  if (!font || injectedFonts.has(font.name)) return;
  const dataUrl = await invoke("get_font_data_url", { filePath: font.file_path });
  const ext = (font.file_path.split(".").pop() || "ttf").toLowerCase();
  const format = ext === "woff2" ? "woff2" : ext === "woff" ? "woff" : ext === "otf" ? "opentype" : "truetype";
  injectFontFace(font.name, dataUrl, format);
  injectedFonts.add(font.name);
}

/** 根据主题模式应用 dark 类 */
function applyTheme(theme) {
  const root = document.documentElement;
  const setDark = (isDark) => {
    root.classList.toggle("dark", isDark);
  };

  if (theme === "dark") {
    setDark(true);
  } else if (theme === "light") {
    setDark(false);
  } else {
    // auto: 跟随系统
    setDark(window.matchMedia("(prefers-color-scheme: dark)").matches);
  }
}

// 监听系统主题变化（仅在 auto 模式下生效）
let mediaListenerBound = false;
let currentTheme = "auto";
function ensureMediaListener() {
  if (mediaListenerBound) return;
  mediaListenerBound = true;
  const mql = window.matchMedia("(prefers-color-scheme: dark)");
  mql.addEventListener("change", (e) => {
    if (currentTheme === "auto") {
      document.documentElement.classList.toggle("dark", e.matches);
    }
  });
}

/** 将设置应用到 CSS 变量与主题 */
export function applySettings(settings) {
  registerBuiltinFonts();
  const root = document.documentElement;

  // 字体大小：仅影响文字本身，不影响组件外壳尺寸
  const fontSize = settings.font_size;
  const sizePx = `${fontSize}px`;
  root.style.setProperty("--font-size", sizePx);
  root.style.setProperty("--el-font-size-base", sizePx);
  root.style.setProperty("--el-font-size-small", `${Math.max(12, fontSize - 2)}px`);
  root.style.setProperty("--el-font-size-large", `${fontSize + 2}px`);

  // 字体族
  root.style.setProperty("--word-font", settings.word_font || "system-ui");
  root.style.setProperty("--phonetic-font", settings.phonetic_font || '"gebinee"');
  root.style.setProperty("--ui-font", settings.ui_font || "system-ui");
  root.style.setProperty("--el-font-family", settings.ui_font || "system-ui");

  // 主题
  currentTheme = settings.theme || "auto";
  ensureMediaListener();
  applyTheme(currentTheme);
}

/** 根据字体名判断是否为内置/系统字体（用于下拉区分） */
export function isSystemFontName(name) {
  return !name || name === "system-ui" || name === "gebinee";
}
