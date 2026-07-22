<script setup>
import { open } from "@tauri-apps/plugin-dialog";
import { reactive, ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { addEntry } from "../api/db";
import { checkEmptyText, checkKeyValidity, emptyMsg, invalidKeyMsg } from "../utils/validation";
import { errorMessage } from "../utils/error";

import { Plus, Upload } from "@element-plus/icons-vue";
import { GebineeButton, GebineeInput } from "@gebinee/components";

const emit = defineEmits(["changed"]);

const form = reactive({ key: "", value: "" });
const adding = ref(false);

let unlistenImportDone = null;

onMounted(async () => {
  unlistenImportDone = await listen("import:done", () => {
    emit("changed");
  });
});

onUnmounted(() => {
  if (unlistenImportDone) unlistenImportDone();
});

async function onAdd() {
  const key = form.key.trim();
  const value = form.value.trim();
  if (checkEmptyText(key)) {
    ElMessage.error(emptyMsg());
    return;
  }
  if (!checkKeyValidity(key)) {
    ElMessage.error(invalidKeyMsg());
    return;
  }
  if (checkEmptyText(value)) {
    ElMessage.error("注音不能为空");
    return;
  }
  adding.value = true;
  try {
    await addEntry(key, value);
    ElMessage.success("添加成功");
    form.key = "";
    form.value = "";
    emit("changed");
  } catch (e) {
    ElMessage.error(errorMessage(e));
  } finally {
    adding.value = false;
  }
}

async function onImportFromExcel() {
  try {
    const path = await open({
      filters: [{ name: "Excel", extensions: ["xlsx", "xls"] }],
      multiple: false,
    });
    if (!path) return;
    await invoke("open_import_window", { filePath: path });
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}
</script>

<template>
  <div class="add-panel">
    <div class="panel-title">添加单词</div>
    <el-form :model="form" label-position="top" @submit.prevent="onAdd">
      <el-form-item class="compact-item">
        <GebineeInput
          v-model="form.key"
          placeholder="单词（仅英文字母和连字符）"
          class="word-input"
          clearable
          @keyup.enter="onAdd"
        />
      </el-form-item>
      <el-form-item class="compact-item">
        <GebineeInput
          v-model="form.value"
          placeholder="注音结果"
          class="phonetic-input"
          clearable
          @keyup.enter="onAdd"
        />
      </el-form-item>
      <div class="actions">
        <GebineeButton class="gebinee--btn-green" :loading="adding" @click="onAdd">
          <el-icon><Plus /></el-icon>
          <span>添加</span>
        </GebineeButton>
        <GebineeButton type="success" plain @click="onImportFromExcel">
          <el-icon><Upload /></el-icon>
          <span>从 Excel 导入</span>
        </GebineeButton>
      </div>
    </el-form>
  </div>
</template>

<style scoped>
.panel-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 20px;
}
.actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}
:deep(.compact-item) {
  margin-bottom: 10px;
}

/*noinspection CssUnusedSymbol*/
:deep(.compact-item .el-form-item__label) {
  padding-bottom: 2px;
  line-height: 1.4;
}

/*noinspection CssUnusedSymbol*/
:deep(.word-input .el-input__inner) {
  font-family: var(--gebinee-word-font), sans-serif;
}

/*noinspection CssUnusedSymbol*/
:deep(.phonetic-input .el-input__inner) {
  font-family: var(--gebinee-phonetic-font), sans-serif;
}
</style>
