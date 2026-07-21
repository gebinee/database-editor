import { invoke } from "@tauri-apps/api/core";

export const initApp = () => invoke("init_app");

export const getSettings = () => invoke("get_settings");

export const saveSettings = (settings) => invoke("save_settings", { settings });

export const pickDatabaseFile = () => invoke("pick_database_file");

export const pickSavePath = (defaultName) => invoke("pick_save_path", { defaultName });

export const pickFontFile = () => invoke("pick_font_file");

export const switchDatabase = (newPath) => invoke("switch_database", { newPath });
