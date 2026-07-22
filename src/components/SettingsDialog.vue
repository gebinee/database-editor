<script setup>
import { reactive, ref, watch, computed } from "vue";
import { pickDatabaseFile, pickFontFile, switchDatabase } from "../api/settings";
import { useSettingsStore } from "../stores/settings";
import { errorMessage } from "../utils/error";

import { SettingsDialog as BaseSettingsDialog } from "@gebinee/components";
import { EditPen } from "@element-plus/icons-vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
});
const emit = defineEmits(["update:visible", "changed"]);

const settingsStore = useSettingsStore();
const saving = ref(false);
const switching = ref(false);

const form = reactive({
  db_path: "",
  word_font: "gebinee",
  phonetic_font: "gebinee",
  ui_font: "system-ui",
  ui_font_cn: "",
  theme: "auto",
  custom_fonts: [],
});

// 外观配置代理：组件库内置 AppearanceTab 通过 v-model:appearance 双向绑定
const appearanceProxy = computed({
  get: () => ({
    word_font: form.word_font,
    phonetic_font: form.phonetic_font,
    ui_font: form.ui_font,
    ui_font_cn: form.ui_font_cn,
    theme: form.theme,
  }),
  set: (v) => Object.assign(form, v),
});

// 数据库配置代理：组件库内置 DatabaseTab 通过 v-model:database 双向绑定
const databaseProxy = computed({
  get: () => ({ db_path: form.db_path }),
  set: (v) => Object.assign(form, v),
});

const fontOptions = computed(() =>
  form.custom_fonts.map(f => ({ label: `${f.name}`, value: f.name }))
);

watch(
  () => props.visible,
  (v) => {
    if (v && settingsStore.settings) {
      Object.assign(form, settingsStore.settings);
    }
  }
);

async function onPickDb() {
  try {
    const path = await pickDatabaseFile();
    if (path) form.db_path = path;
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}

async function onPickFontFile() {
  try {
    const font = await pickFontFile();
    if (!form.custom_fonts.some((f) => f.name === font.name)) {
      form.custom_fonts.push(font);
    }
    ElMessage.success(`已添加字体：${font.name}`);
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}

async function onSave() {
  saving.value = true;
  try {
    const oldPath = settingsStore.settings?.db_path;
    await settingsStore.save({ ...form });
    // 若数据库路径变更，切换数据库
    if (oldPath && oldPath !== form.db_path) {
      switching.value = true;
      try {
        await switchDatabase(form.db_path);
        ElMessage.success("数据库已切换");
      } catch (e) {
        ElMessage.error(`数据库切换失败：${errorMessage(e)}`);
      } finally {
        switching.value = false;
      }
    }
    emit("changed");
    emit("update:visible", false);
    ElMessage.success("设置已保存");
  } catch (e) {
    ElMessage.error(errorMessage(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <BaseSettingsDialog
    :visible="visible"
    @update:visible="emit('update:visible', $event)"
    v-model:appearance="appearanceProxy"
    v-model:database="databaseProxy"
    :font-options="fontOptions"
    :show-database-tab="true"
    :show-appearance-tab="true"
    :show-update-button="true"
    :app-icon="EditPen"
    app-name="gebinee 单词数据库编辑器"
    :saving="saving || switching"
    @save="onSave"
    @pick-database-file="onPickDb"
    @pick-font-file="onPickFontFile"
  />
</template>
