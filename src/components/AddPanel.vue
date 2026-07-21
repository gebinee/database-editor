<script setup>
import { reactive, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { addEntry } from "../api/db";
import {
  checkEmptyText,
  checkKeyValidity,
  emptyMsg,
  invalidKeyMsg,
} from "../utils/validation";
import { errorMessage } from "../utils/error";
import ImportDialog from "./ImportDialog.vue";
import { Plus, Upload } from "@element-plus/icons-vue";

const emit = defineEmits(["changed"]);

const form = reactive({ key: "", value: "" });
const adding = ref(false);
const importDialogVisible = ref(false);
const importFilePath = ref("");

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
    importFilePath.value = path;
    importDialogVisible.value = true;
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}

function onImported() {
  emit("changed");
}
</script>

<template>
  <div class="add-panel">
    <div class="panel-title">添加单词</div>
    <el-form :model="form" label-position="top" @submit.prevent="onAdd">
      <el-form-item class="compact-item">
        <el-input
          v-model="form.key"
          placeholder="单词（仅英文字母和连字符）"
          class="font-word input-lg"
          clearable
          @keyup.enter="onAdd"
        />
      </el-form-item>
      <el-form-item class="compact-item">
        <el-input
          v-model="form.value"
          placeholder="注音结果"
          class="font-phonetic input-lg"
          clearable
          @keyup.enter="onAdd"
        />
      </el-form-item>
      <div class="actions">
        <el-button class="btn-green btn-side" :loading="adding" @click="onAdd">
          <el-icon><Plus /></el-icon>
          <span>添加</span>
        </el-button>
        <el-button type="success" plain class="btn-side btn-success-hover" @click="onImportFromExcel">
          <el-icon><Upload /></el-icon>
          <span>从 Excel 导入</span>
        </el-button>
      </div>
    </el-form>

    <ImportDialog
      v-model:visible="importDialogVisible"
      :file-path="importFilePath"
      @imported="onImported"
    />
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
:deep(.btn-side) {
  height: 38px;
}
:deep(.compact-item) {
  margin-bottom: 10px;
}

/*noinspection CssUnusedSymbol*/
:deep(.compact-item .el-form-item__label) {
  padding-bottom: 2px;
  line-height: 1.4;
}
</style>
