<script setup>
import { reactive, ref } from "vue";
import { getEntry, deleteEntry } from "../api/db";
import {
  checkEmptyText,
  checkKeyValidity,
  emptyMsg,
  invalidKeyMsg,
} from "../utils/validation";
import { errorMessage } from "../utils/error";
import EditDialog from "./EditDialog.vue";
import { Close, Delete, Edit, Search } from "@element-plus/icons-vue";

const emit = defineEmits(["changed"]);

const form = reactive({ key: "" });
const loading = ref(false);
const result = ref(null); // { key, value } | null
const searched = ref(false);
const editVisible = ref(false);
const editingEntry = ref(null);

async function onQuery() {
  const key = form.key.trim();
  if (checkEmptyText(key)) {
    ElMessage.error(emptyMsg());
    return;
  }
  if (!checkKeyValidity(key)) {
    ElMessage.error(invalidKeyMsg());
    return;
  }
  loading.value = true;
  try {
    const entry = await getEntry(key);
    result.value = entry;
    searched.value = true;
    if (!entry) {
      ElMessage.warning("未找到该单词");
    }
  } catch (e) {
    ElMessage.error(errorMessage(e));
  } finally {
    loading.value = false;
  }
}

function onClear() {
  form.key = "";
  result.value = null;
  searched.value = false;
}

function openEdit() {
  if (!result.value) return;
  editingEntry.value = { ...result.value };
  editVisible.value = true;
}

async function onDeleteResult() {
  if (!result.value) return;
  try {
    await ElMessageBox.confirm(`确认删除单词 "${result.value.key}" 吗？`, "删除确认", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
  } catch {
    return;
  }
  try {
    await deleteEntry(result.value.key);
    ElMessage.success("删除成功");
    result.value = null;
    searched.value = false;
    emit("changed");
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}

function onEditChanged() {
  emit("changed");
  onQuery();
}
</script>

<template>
  <div class="query-panel">
    <div class="panel-title">查询单词</div>
    <el-form :model="form" label-position="top" @submit.prevent="onQuery">
      <el-form-item class="compact-item">
        <el-input
          v-model="form.key"
          placeholder="输入完整单词查询"
          class="font-word input-lg"
          clearable
          @keyup.enter="onQuery"
        />
      </el-form-item>
      <div class="query-actions">
        <el-button type="primary" class="btn-side" :loading="loading" @click="onQuery">
          <el-icon><Search /></el-icon>
          <span>查询</span>
        </el-button>
        <el-button type="warning" plain class="btn-side" @click="onClear">
          <el-icon><Close /></el-icon>
          <span>清除</span>
        </el-button>
      </div>
    </el-form>

    <el-card v-if="searched" class="result-card" shadow="never">
      <template v-if="result">
        <div class="result-row">
          <span class="label">单词</span>
          <span class="font-word result-value">{{ result.key }}</span>
        </div>
        <div class="result-row">
          <span class="label">注音</span>
          <span class="font-phonetic result-value">{{ result.value }}</span>
        </div>
        <div class="result-actions">
          <el-button type="success" plain class="btn-side btn-success-hover" @click="openEdit">
            <el-icon><Edit /></el-icon>
            <span>编辑</span>
          </el-button>
          <el-button type="danger" plain class="btn-side" @click="onDeleteResult">
            <el-icon><Delete /></el-icon>
            <span>删除</span>
          </el-button>
        </div>
      </template>
      <el-empty v-else description="未找到该单词" :image-size="48" class="empty-tip" />
    </el-card>

    <EditDialog
      v-model:visible="editVisible"
      :entry="editingEntry"
      @changed="onEditChanged"
    />
  </div>
</template>

<style scoped>
.query-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
}
.panel-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 20px;
}
.result-card {
  margin-top: 20px;
}
.result-row {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 7px 0;
}
.result-row .label {
  width: 42px;
  color: var(--el-text-color-secondary);
  font-size: 16px;
  flex-shrink: 0;
}
.result-value {
  font-size: 18px;
  word-break: break-all;
}
.result-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

/*noinspection CssUnusedSymbol*/
:deep(.empty-tip .el-empty__description) {
  font-size: 16px;
}
.query-actions {
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

/*noinspection CssUnusedSymbol*/
:deep(.result-card .el-card__body) {
  padding: 10px 12px;
}
</style>
