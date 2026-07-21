import { createApp } from "vue";
import { createPinia } from "pinia";
import "element-plus/theme-chalk/dark/css-vars.css";
import App from "./App.vue";
import "./styles/main.css";

const app = createApp(App);
app.use(createPinia());
app.mount("#app");
