import { defineStore } from "pinia";
import { ref } from "vue";
import * as settingsApi from "../api/settings";
import { applyAppearance, registerCustomFonts } from "@gebinee/components";

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
      await registerCustomFonts(settings.value.custom_fonts);
    }
    applyAppearance(settings.value);
    ready.value = true;
  }

  async function save(newSettings) {
    await settingsApi.saveSettings(newSettings);
    settings.value = newSettings;
    if (newSettings.custom_fonts) {
      await registerCustomFonts(newSettings.custom_fonts);
    }
    applyAppearance(newSettings);
  }

  function clearDbMissing() {
    dbMissing.value = false;
  }

  return { settings, dbError, dbMissing, ready, init, save, clearDbMissing };
});
