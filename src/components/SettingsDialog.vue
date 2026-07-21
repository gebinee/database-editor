<script setup>
import { reactive, ref, watch, computed } from "vue";
import { useSettingsStore } from "../stores/settings";
import {
  pickDatabaseFile,
  pickFontFile,
  switchDatabase,
} from "../api/settings";
import { errorMessage } from "../utils/error";
import { SettingsDialog as BaseSettingsDialog } from "@gebinee/components";
import { Brush, Coin, EditPen, FolderOpened, Upload } from "@element-plus/icons-vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
});
const emit = defineEmits(["update:visible", "changed"]);

const settingsStore = useSettingsStore();
const saving = ref(false);
const switching = ref(false);

const form = reactive({
  db_path: "",
  font_size: 14,
  word_font: "system-ui",
  phonetic_font: "gebinee",
  ui_font: "system-ui",
  theme: "auto",
  custom_fonts: [],
});

// 项目特有的 tab 定义（"关于"tab 由包内置）
const tabs = [
  { name: "database", label: "数据库", icon: Coin },
  { name: "appearance", label: "外观", icon: Brush },
];

const fontOptions = computed(() => {
  const builtin = [
    { label: "系统字体", value: "system-ui" },
    { label: "gebinee", value: "gebinee" },
  ];
  const custom = form.custom_fonts.map((f) => ({
    label: `${f.name}`,
    value: f.name,
  }));
  return [...builtin, ...custom];
});

const themeOptions = [
  { label: "浅色模式", value: "light" },
  { label: "深色模式", value: "dark" },
  { label: "跟随系统", value: "auto" },
];

watch(
  () => props.visible,
  (v) => {
    if (v && settingsStore.settings) {
      Object.assign(form, settingsStore.settings);
      // 兼容旧 dark_mode 字段
      if (!form.theme) {
        form.theme = form.dark_mode ? "dark" : "auto";
      }
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
    :app-name="'图形化数据库编辑工具'"
    :app-icon="EditPen"
    :tabs="tabs"
    :saving="saving || switching"
    @save="onSave"
  >
    <!-- 数据库 tab：项目特有 -->
    <template #tab-database>
      <el-form :model="form" label-position="top">
        <el-form-item label="数据库文件路径">
          <div class="db-row">
            <el-input v-model="form.db_path" placeholder="数据库文件路径" />
            <el-button @click="onPickDb">
              <el-icon><FolderOpened /></el-icon>
              <span>选择</span>
            </el-button>
          </div>
        </el-form-item>
      </el-form>
    </template>

    <!-- 外观 tab：项目特有 -->
    <template #tab-appearance>
      <el-form :model="form" label-position="top">
        <el-divider content-position="left">主题</el-divider>
        <el-form-item label="主题模式">
          <el-radio-group v-model="form.theme">
            <el-radio-button
              v-for="o in themeOptions"
              :key="o.value"
              :value="o.value"
            >
              {{ o.label }}
            </el-radio-button>
          </el-radio-group>
        </el-form-item>

        <el-divider content-position="left">字体</el-divider>
        <el-form-item label="单词字体">
          <el-select v-model="form.word_font" style="width: 100%">
            <el-option
              v-for="o in fontOptions"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="注音字体">
          <el-select v-model="form.phonetic_font" style="width: 100%">
            <el-option
              v-for="o in fontOptions"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="UI 字体">
          <el-select v-model="form.ui_font" style="width: 100%">
            <el-option
              v-for="o in fontOptions"
              :key="o.value"
              :label="o.label"
              :value="o.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button @click="onPickFontFile">
            <el-icon><Upload /></el-icon>
            <span>上传字体文件</span>
          </el-button>
          <span class="hint">支持 ttf/otf/woff/woff2</span>
        </el-form-item>
      </el-form>
    </template>
  </BaseSettingsDialog>
</template>

<style scoped>
.db-row {
  display: flex;
  gap: 8px;
  width: 100%;
}

/*noinspection CssUnusedSymbol*/
.db-row .el-input {
  flex: 1;
}
.hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  margin-left: 8px;
}
</style>
