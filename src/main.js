import { createApp } from "vue";
import { createPinia } from "pinia";
import "element-plus/theme-chalk/dark/css-vars.css";
import "@gebinee/components/style.css";
import App from "./App.vue";
import { registerFontLoader } from "@gebinee/components";
import { invoke } from "@tauri-apps/api/core";

// 注册字体加载器：组件库的 registerCustomFont 通过它读取字体文件 data URL
registerFontLoader((filePath) => invoke("get_font_data_url", { filePath }));

const app = createApp(App);
app.use(createPinia());
app.mount("#app");
