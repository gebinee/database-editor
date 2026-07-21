import { defineStore } from "pinia";
import { ref } from "vue";
import * as settingsApi from "../api/settings";
import { applySettings, registerCustomFont } from "../utils/font";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref(null);
  const dbError = ref(null);
  const dbMissing = ref(false);
  const ready = ref(false);

  async function init() {
    const info = await settingsApi.initApp();
    settings.value = info.settings;
    dbError.value = info.db_error || null;
    dbMissing.value = !!info.db_missing;
    if (settings.value && settings.value.custom_fonts) {
      for (const f of settings.value.custom_fonts) {
        try {
          await registerCustomFont(f);
        } catch (e) {
          console.error("注册字体失败:", f.name, e);
        }
      }
    }
    applySettings(settings.value);
    ready.value = true;
  }

  async function save(newSettings) {
    await settingsApi.saveSettings(newSettings);
    settings.value = newSettings;
    if (newSettings.custom_fonts) {
      for (const f of newSettings.custom_fonts) {
        try {
          await registerCustomFont(f);
        } catch (e) {
          console.error("注册字体失败:", f.name, e);
        }
      }
    }
    applySettings(newSettings);
  }

  function clearDbError() {
    dbError.value = null;
  }

  function clearDbMissing() {
    dbMissing.value = false;
  }

  return { settings, dbError, dbMissing, ready, init, save, clearDbError, clearDbMissing };
});
