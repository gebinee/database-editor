import { createApp, h } from "vue";
import { createPinia } from "pinia";
import { ElConfigProvider } from "element-plus";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import "element-plus/theme-chalk/dark/css-vars.css";
import "@gebinee/components/style.css";
import ImportView from "./views/ImportView.vue";
import { registerFontLoader, applyAppearance, registerCustomFonts } from "@gebinee/components";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

console.log("[import-main] 开始加载导入窗口");

registerFontLoader((filePath) => invoke("get_font_data_url", { filePath }));

// Element Plus 中文本地化（自定义 pagination goto 文案，与主窗口保持一致）
const customLocale = {
  ...zhCn,
  el: {
    ...zhCn.el,
    pagination: {
      ...zhCn.el.pagination,
      goto: "跳转到",
    },
  },
};

// 移除 loading 提示
function clearLoading() {
  const el = document.getElementById("loading-fallback");
  if (el) el.remove();
}

// 显示错误信息
function showError(msg) {
  clearLoading();
  let el = document.getElementById("error-fallback");
  if (!el) {
    el = document.createElement("div");
    el.id = "error-fallback";
    document.body.appendChild(el);
  }
  el.textContent = "加载失败: " + msg;
}

// 应用软件设置（主题、字体等）——与主窗口 App.vue 的 onMounted 逻辑保持一致
async function applySettings() {
  try {
    const info = await invoke("init_app");
    if (info.settings) {
      if (info.settings.custom_fonts) {
        await registerCustomFonts(info.settings.custom_fonts);
      }
      applyAppearance(info.settings);
      console.log("[import-main] 软件设置已应用");
    }
  } catch (e) {
    console.error("[import-main] 应用设置失败:", e);
  }
}

try {
  console.log("[import-main] 创建 Vue 应用");
  // 用 ElConfigProvider 包裹 ImportView，使中文 locale 生效
  const App = {
    render() {
      return h(ElConfigProvider, { locale: customLocale }, () => h(ImportView));
    },
  };
  const app = createApp(App);
  app.use(createPinia());
  app.config.errorHandler = (err, instance, info) => {
    console.error("[ImportView] Vue error:", err, info);
    showError(`Vue 渲染错误: ${err?.message || err}\n\n${err?.stack || ""}`);
  };

  // 先应用主题、字体等软件设置，再挂载应用，避免初始闪烁
  applySettings().finally(() => {
    console.log("[import-main] 挂载应用");
    app.mount("#app");
    console.log("[import-main] 挂载成功");
    clearLoading();
  });

  // 监听设置变更事件，主窗口保存设置后会 emit，重新应用主题/字体
  listen("settings:changed", () => {
    console.log("[import-main] 收到设置变更事件，重新应用设置");
    applySettings();
  });
} catch (e) {
  console.error("[import-main] 挂载失败:", e);
  showError(`${e?.message || e}\n\n${e?.stack || ""}`);
}
