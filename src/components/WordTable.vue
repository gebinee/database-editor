<script setup>
import { ref, watch, onMounted } from "vue";
import { listEntries, deleteEntry } from "../api/db";
import { useQueryStore } from "../stores/query";
import { errorMessage } from "../utils/error";
import EditDialog from "./EditDialog.vue";
import { Delete, Edit } from "@element-plus/icons-vue";

const emit = defineEmits(["changed"]);

const queryStore = useQueryStore();
const loading = ref(false);
const items = ref([]);
const total = ref(0);
const editVisible = ref(false);
const editingEntry = ref(null);

async function fetchData() {
  loading.value = true;
  try {
    const res = await listEntries({
      search: queryStore.search || null,
      page: queryStore.page,
      pageSize: queryStore.pageSize,
      sortOrder: queryStore.sortOrder,
    });
    items.value = res.items;
    total.value = res.total;
  } catch (e) {
    ElMessage.error(errorMessage(e));
    items.value = [];
    total.value = 0;
  } finally {
    loading.value = false;
  }
}

function onSearch() {
  queryStore.resetToFirstPage();
  fetchData();
}

function onClear() {
  queryStore.search = "";
  queryStore.resetToFirstPage();
  fetchData();
}

function onRefresh() {
  fetchData();
}

function onSortChange({ order }) {
  queryStore.sortOrder = order === "descending" ? "desc" : "asc";
  fetchData();
}

function onPageChange(p) {
  queryStore.page = p;
  fetchData();
}

function onPageSizeChange(size) {
  queryStore.pageSize = size;
  queryStore.resetToFirstPage();
  fetchData();
}

function openEdit(row) {
  editingEntry.value = { ...row };
  editVisible.value = true;
}

async function onDeleteRow(row) {
  try {
    await ElMessageBox.confirm(`确认删除单词 "${row.key}" 吗？`, "删除确认", {
      type: "warning",
      confirmButtonText: "删除",
      cancelButtonText: "取消",
    });
  } catch {
    return;
  }
  try {
    await deleteEntry(row.key);
    ElMessage.success("删除成功");
    emit("changed");
    fetchData();
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}

function onEditChanged() {
  emit("changed");
  fetchData();
}

onMounted(fetchData);

watch(() => queryStore.sortOrder, fetchData);

// 暴露刷新方法供父组件调用
defineExpose({ refresh: fetchData });
</script>

<template>
  <div class="word-table">
    <div class="toolbar">
      <el-input
        v-model="queryStore.search"
        placeholder="搜索单词…"
        clearable
        class="search-input font-word input-lg"
        @keyup.enter="onSearch"
        @clear="onSearch"
      />
      <div class="toolbar-actions">
        <el-button type="primary" plain class="btn-tall" @click="onSearch">搜索</el-button>
        <el-button type="warning" plain class="btn-tall" @click="onClear">清除</el-button>
        <el-button class="btn-tall" @click="onRefresh">刷新</el-button>
      </div>
    </div>

    <el-table
      :data="items"
      v-loading="loading"
      height="100%"
      border
      stripe
      @sort-change="onSortChange"
      empty-text="暂无数据"
    >
      <el-table-column
        prop="key"
        label="单词"
        sortable="custom"
        min-width="200"
        align="center"
        header-align="center"
      >
        <template #default="{ row }">
          <span class="font-word cell-text">{{ row.key }}</span>
        </template>
      </el-table-column>
      <el-table-column
        prop="value"
        label="注音结果"
        min-width="200"
        align="center"
        header-align="center"
        show-overflow-tooltip
      >
        <template #default="{ row }">
          <span class="font-phonetic cell-text">{{ row.value }}</span>
        </template>
      </el-table-column>
      <el-table-column label="操作" width="200" align="center" fixed="right">
        <template #default="{ row }">
          <div class="op-cell">
            <el-button type="success" plain class="btn-cell btn-success-hover" @click="openEdit(row)">
              <el-icon><Edit /></el-icon>
              <span>编辑</span>
            </el-button>
            <el-button type="danger" plain class="btn-cell" @click="onDeleteRow(row)">
              <el-icon><Delete /></el-icon>
              <span>删除</span>
            </el-button>
          </div>
        </template>
      </el-table-column>
    </el-table>

    <div class="pager">
      <span class="pager-total">共 {{ total }} 个单词</span>
      <div class="page-size-wrap">
        <el-select
          v-model="queryStore.pageSize"
          size="small"
          class="page-size-select"
          @change="onPageSizeChange"
        >
          <el-option
            v-for="s in [10, 20, 30, 40, 50]"
            :key="s"
            :label="String(s)"
            :value="s"
          />
        </el-select>
        <span class="page-unit">条/页</span>
      </div>
      <el-pagination
        v-model:current-page="queryStore.page"
        :total="total"
        layout="prev, pager, next, jumper"
        background
        @current-change="onPageChange"
      />
    </div>

    <EditDialog
      v-model:visible="editVisible"
      :entry="editingEntry"
      @changed="onEditChanged"
    />
  </div>
</template>

<style scoped>
.word-table {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}
.toolbar {
  display: flex;
  gap: 16px;
  align-items: center;
  padding-bottom: 8px;
  flex-shrink: 0;
}
.search-input {
  flex: 1;
  max-width: 340px;
}
.toolbar-actions {
  display: flex;
  gap: 4px;
}
:deep(.btn-tall) {
  height: 40px;
}

/*noinspection CssUnusedSymbol*/
:deep(.el-table th.el-table__cell > .cell) {
  font-size: 16px;
}
:deep(.btn-cell) {
  font-size: 15px;
  padding: 8px 10px;
}
.op-cell {
  display: flex;
  justify-content: center;
  gap: 8px;
}
.cell-text {
  word-break: break-all;
  font-size: 18px;
}
.pager {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding-top: 8px;
}
.pager-total {
  font-size: 14px;
  color: var(--el-text-color-regular);
}
.page-size-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}
.page-size-select {
  width: 80px;
}
.page-unit {
  font-size: 14px;
  color: var(--el-text-color-regular);
}

/*noinspection CssUnusedSymbol*/
:deep(.el-pagination__jump) {
  font-size: 14px !important;
}

/*noinspection CssUnusedSymbol*/
:deep(.el-pagination__jump .el-input__inner) {
  font-size: 14px !important;
}
</style>
