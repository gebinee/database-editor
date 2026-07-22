<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import {
  readExcelForImport,
  checkDuplicates,
  importEntries,
  exportProblemWords,
} from "../api/db";
import { pickSavePath } from "../api/settings";
import { checkEmptyText, checkKeyValidity } from "../utils/validation";
import { errorMessage } from "../utils/error";
import { GebineeButton } from "@gebinee/components";
import {
  Download,
  Edit,
  Delete,
  RefreshLeft,
  Check,
  Close,
  WarningFilled,
} from "@element-plus/icons-vue";

const pageSize = ref(50);
const loading = ref(false);
const importing = ref(false);
const rows = ref([]);
const fileName = ref("");
const dbExistsCache = ref(new Map());
const highlightIndex = ref(null);
const onlyProblems = ref(false);
const page = ref(1);
const tableRef = ref();
const showProblems = ref(false);

// 从后端获取待导入路径并加载
async function loadPendingPath() {
  try {
    const path = await invoke("take_pending_import_path");
    if (path) {
      await load(path);
    }
  } catch (e) {
    console.error("[ImportView] loadPendingPath error:", e);
  }
}

let unlistenImportOpen = null;
onMounted(async () => {
  try {
    // 首次挂载：通过状态变量获取路径（动态创建窗口的场景）
    await loadPendingPath();
    // 后续导入：通过事件获取新路径（窗口已存在的场景）
    unlistenImportOpen = await listen("import:open", () => {
      loadPendingPath();
    });
  } catch (e) {
    console.error("[ImportView] onMounted error:", e);
  }
});

onUnmounted(() => {
  if (unlistenImportOpen) unlistenImportOpen();
});

function reset() {
  rows.value = [];
  fileName.value = "";
  dbExistsCache.value = new Map();
  highlightIndex.value = null;
  onlyProblems.value = false;
  page.value = 1;
  showProblems.value = false;
}

async function load(filePath) {
  loading.value = true;
  try {
    fileName.value = filePath.split(/[\\/]/).pop() || filePath;
    const raw = await readExcelForImport(filePath);
    rows.value = raw.map((r) => ({
      index: r.index,
      key: r.key,
      value: r.value,
      status: "danger",
      problems: [],
      editing: false,
      draftKey: "",
      draftValue: "",
    }));
    recomputeLocal();
    const validKeys = [
      ...new Set(
        rows.value
          .filter((r) => r.key && checkKeyValidity(r.key))
          .map((r) => r.key)
      ),
    ];
    if (validKeys.length > 0) {
      const existsArr = await checkDuplicates(validKeys);
      validKeys.forEach((k, i) => dbExistsCache.value.set(k, !!existsArr[i]));
    }
    recomputeStatus();
    page.value = 1;
  } catch (e) {
    ElMessage.error(errorMessage(e));
    closeWindow();
  } finally {
    loading.value = false;
  }
}

function recomputeLocal() {
  const seen = new Set();
  rows.value = rows.value.filter((r) => {
    const id = `${r.key}\x00${r.value}`;
    if (seen.has(id)) return false;
    seen.add(id);
    return true;
  });

  const countMap = new Map();
  for (const r of rows.value) {
    r.problems = [];
    if (checkEmptyText(r.key)) {
      r.problems.push("单词为空");
    } else if (!checkKeyValidity(r.key)) {
      r.problems.push("单词含非法字符");
    }
    if (checkEmptyText(r.value)) {
      r.problems.push("注音为空");
    }
    if (r.key && checkKeyValidity(r.key)) {
      countMap.set(r.key, (countMap.get(r.key) || 0) + 1);
    }
  }
  for (const r of rows.value) {
    if (r.key && checkKeyValidity(r.key) && (countMap.get(r.key) || 0) > 1) {
      r.problems.push("Excel内重复");
    }
  }
}

function recomputeStatus() {
  for (const r of rows.value) {
    const dbExists =
      !!r.key &&
      checkKeyValidity(r.key) &&
      dbExistsCache.value.get(r.key) === true;
    if (dbExists && !r.problems.includes("单词已存在")) {
      r.problems.push("单词已存在");
    }
    if (!dbExists) {
      r.problems = r.problems.filter((p) => p !== "单词已存在");
    }
    if (r.status === "ignored") continue;
    if (r.problems.length === 0) {
      r.status = "success";
    } else if (
      r.problems.length === 1 &&
      r.problems[0] === "单词已存在"
    ) {
      r.status = "warning";
    } else {
      r.status = "danger";
    }
  }
}

const problemRows = computed(() =>
  rows.value.filter((r) => r.status === "danger" || r.status === "warning")
);
const successCount = computed(
  () => rows.value.filter((r) => r.status === "success").length
);
const warningCount = computed(
  () => rows.value.filter((r) => r.status === "warning").length
);
const dangerCount = computed(
  () => rows.value.filter((r) => r.status === "danger").length
);
const ignoredCount = computed(
  () => rows.value.filter((r) => r.status === "ignored").length
);
const filteredRows = computed(() => {
  if (!onlyProblems.value) return rows.value;
  return rows.value.filter(
    (r) => r.status === "danger" || r.status === "warning"
  );
});
const pagedRows = computed(() => {
  const start = (page.value - 1) * pageSize.value;
  return filteredRows.value.slice(start, start + pageSize.value);
});
const totalFiltered = computed(() => filteredRows.value.length);

function onPageSizeChange() {
  page.value = 1;
}

function onPageChange(p) {
  page.value = p;
}

function startEdit(row) {
  for (const r of rows.value) {
    if (r.editing && r !== row) cancelEdit(r);
  }
  row.editing = true;
  row.draftKey = row.key;
  row.draftValue = row.value;
}

function cancelEdit(row) {
  row.editing = false;
  row.draftKey = "";
  row.draftValue = "";
}

async function saveEdit(row) {
  row.key = (row.draftKey || "").trim();
  row.value = (row.draftValue || "").trim();
  row.editing = false;
  row.draftKey = "";
  row.draftValue = "";
  recomputeLocal();
  if (row.key && checkKeyValidity(row.key)) {
    try {
      const [exists] = await checkDuplicates([row.key]);
      dbExistsCache.value.set(row.key, !!exists);
    } catch (e) {
      // ignore
    }
  }
  recomputeStatus();
}

async function locateRow(index) {
  let pos = filteredRows.value.findIndex((r) => r.index === index);
  if (pos < 0) {
    onlyProblems.value = false;
    await nextTick();
    pos = filteredRows.value.findIndex((r) => r.index === index);
    if (pos < 0) return;
  }
  page.value = Math.floor(pos / pageSize.value) + 1;
  const row = filteredRows.value[pos];
  highlightIndex.value = index;
  await nextTick();
  tableRef.value?.setCurrentRow?.(row);
  await nextTick();
  const tableEl = tableRef.value?.$el;
  const el = tableEl?.querySelector(".el-table__body .current-row");
  el?.scrollIntoView({ behavior: "smooth", block: "center" });
  startEdit(row);
  setTimeout(() => {
    if (highlightIndex.value === index) highlightIndex.value = null;
  }, 1600);
}

function discardRow(index) {
  const r = rows.value.find((x) => x.index === index);
  if (r) r.status = "ignored";
}

function restoreRow(index) {
  const r = rows.value.find((x) => x.index === index);
  if (!r) return;
  r.status = "danger";
  recomputeLocal();
  recomputeStatus();
}

function rowClassName({ row }) {
  const cls = [];
  if (row.status === "danger") cls.push("row-danger");
  if (row.status === "warning") cls.push("row-warning");
  if (row.status === "ignored") cls.push("row-ignored");
  if (row.index === highlightIndex.value) cls.push("row-highlight");
  return cls.join(" ");
}

async function exportProblems() {
  try {
    const path = await pickSavePath("问题单词.xlsx");
    if (!path) return;
    const items = problemRows.value.map((r) => ({
      key: r.key,
      value: r.value,
      problem: r.problems.join("；"),
    }));
    await exportProblemWords(items, path);
    ElMessage.success("已导出问题单词");
  } catch (e) {
    ElMessage.error(errorMessage(e));
  }
}

async function importAll() {
  const successRows = rows.value.filter((r) => r.status === "success");
  if (successRows.length === 0) return;
  importing.value = true;
  try {
    const items = successRows.map((r) => ({ key: r.key, value: r.value }));
    const result = await importEntries(items);
    if (result.inserted > 0) {
      ElMessage.success(`成功导入 ${result.inserted} 条`);
    }
    if (result.problems && result.problems.length > 0) {
      try {
        await ElMessageBox.confirm(
          `有 ${result.problems.length} 条因竞态未导入，是否导出这些问题项？`,
          "导入提示",
          {
            type: "warning",
            confirmButtonText: "导出",
            cancelButtonText: "取消",
          }
        );
        const path = await pickSavePath("竞态问题单词.xlsx");
        if (path) {
          await exportProblemWords(result.problems, path);
          ElMessage.success("已导出竞态问题项");
        }
      } catch {
        // 用户取消
      }
    }
    await emit("import:done");
    closeWindow();
  } catch (e) {
    ElMessage.error(errorMessage(e));
  } finally {
    importing.value = false;
  }
}

function closeWindow() {
  // 隐藏窗口而不是关闭，避免动态创建窗口时的白屏问题
  getCurrentWindow().hide();
}

const STATUS_TYPE = {
  success: "success",
  warning: "warning",
  danger: "danger",
  ignored: "info",
};
const STATUS_TEXT = {
  success: "可导入",
  warning: "已存在",
  danger: "有问题",
  ignored: "已放弃",
};
function tagType(s) {
  return STATUS_TYPE[s];
}
function tagText(s) {
  return STATUS_TEXT[s];
}
</script>

<template>
  <div v-loading="loading" class="import-page">
    <div class="toolbar">
      <div class="title-row">
        <span class="panel-title">数据预览</span>
        <span v-if="fileName" class="file-name">· {{ fileName }}</span>
      </div>
      <span class="count">共 {{ rows.length }} 行</span>
      <div class="spacer" />
      <span class="switch-label">只看问题项</span>
      <el-switch v-model="onlyProblems" />
      <el-button
        :type="problemRows.length > 0 ? 'warning' : 'default'"
        plain
        class="problem-toggle"
        @click="showProblems = !showProblems"
      >
        <el-icon><WarningFilled /></el-icon>
        <span>问题单词 ({{ problemRows.length }})</span>
      </el-button>
    </div>

    <div class="table-wrap">
      <el-table
        ref="tableRef"
        :data="pagedRows"
        :row-class-name="rowClassName"
        height="100%"
        @row-dblclick="startEdit"
        border
        empty-text="暂无数据"
      >
        <el-table-column
          label="#"
          prop="index"
          width="70"
          align="center"
        />
        <el-table-column
          label="单词"
          min-width="170"
          align="center"
          header-align="center"
        >
          <template #default="{ row }">
            <el-input
              v-if="row.editing"
              v-model="row.draftKey"
              size="small"
              placeholder="单词"
              @keyup.enter="saveEdit(row)"
              @keyup.esc="cancelEdit(row)"
            />
            <span v-else class="cell-text word-text">{{
              row.key || "—"
            }}</span>
          </template>
        </el-table-column>
        <el-table-column
          label="注音"
          min-width="190"
          align="center"
          header-align="center"
        >
          <template #default="{ row }">
            <el-input
              v-if="row.editing"
              v-model="row.draftValue"
              size="small"
              placeholder="注音"
              @keyup.enter="saveEdit(row)"
              @keyup.esc="cancelEdit(row)"
            />
            <span v-else class="cell-text phonetic-text">{{
              row.value || "—"
            }}</span>
          </template>
        </el-table-column>
        <el-table-column
          label="状态"
          width="110"
          align="center"
        >
          <template #default="{ row }">
            <el-tag
              :type="tagType(row.status)"
              effect="light"
            >
              {{ tagText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          label="操作"
          width="200"
          align="center"
          fixed="right"
        >
          <template #default="{ row }">
            <div class="op-cell">
              <template v-if="row.editing">
                <el-button
                  type="success"
                  plain
                  class="btn-cell gebinee--btn-success-hover"
                  @click="saveEdit(row)"
                >
                  <el-icon><Check /></el-icon>
                  <span>保存</span>
                </el-button>
                <el-button
                  type="danger"
                  plain
                  class="btn-cell"
                  @click="cancelEdit(row)"
                >
                  <el-icon><Close /></el-icon>
                  <span>取消</span>
                </el-button>
              </template>
              <template v-else>
                <el-button
                  type="success"
                  plain
                  class="btn-cell gebinee--btn-success-hover"
                  @click="startEdit(row)"
                >
                  <el-icon><Edit /></el-icon>
                  <span>编辑</span>
                </el-button>
                <el-button
                  v-if="row.status !== 'ignored'"
                  type="danger"
                  plain
                  class="btn-cell"
                  @click="discardRow(row.index)"
                >
                  <el-icon><Delete /></el-icon>
                  <span>放弃</span>
                </el-button>
                <el-button
                  v-else
                  type="success"
                  plain
                  class="btn-cell"
                  @click="restoreRow(row.index)"
                >
                  <el-icon><RefreshLeft /></el-icon>
                  <span>恢复</span>
                </el-button>
              </template>
            </div>
          </template>
        </el-table-column>
      </el-table>
    </div>
    <div class="pager">
      <span class="pager-total">共 {{ totalFiltered }} 条</span>
      <div class="page-size-wrap">
        <el-select
          v-model="pageSize"
          size="small"
          class="page-size-select"
          @change="onPageSizeChange"
        >
          <el-option
            v-for="s in [10, 20, 30, 40, 50, 100]"
            :key="s"
            :label="String(s)"
            :value="s"
          />
        </el-select>
        <span class="page-unit">条/页</span>
      </div>
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="totalFiltered"
        layout="prev, pager, next, jumper"
        background
        @current-change="onPageChange"
      />
    </div>

    <!-- 问题面板覆盖层 -->
    <Transition name="overlay">
      <div v-if="showProblems" class="problem-overlay" @click.self="showProblems = false">
        <div class="problem-panel">
          <div class="panel-header">
            <span class="panel-title">问题单词</span>
            <span class="problem-count">（{{ problemRows.length }}）</span>
            <el-button
              :icon="Close"
              text
              class="panel-close"
              @click="showProblems = false"
            />
          </div>
          <div class="stats">
            <div class="stat-item success">
              <div class="stat-num">{{ successCount }}</div>
              <div class="stat-label">可导入</div>
            </div>
            <div class="stat-item warning">
              <div class="stat-num">{{ warningCount }}</div>
              <div class="stat-label">已存在</div>
            </div>
            <div class="stat-item danger">
              <div class="stat-num">{{ dangerCount }}</div>
              <div class="stat-label">有问题</div>
            </div>
            <div class="stat-item ignored">
              <div class="stat-num">{{ ignoredCount }}</div>
              <div class="stat-label">已放弃</div>
            </div>
          </div>
          <div class="problem-list">
            <el-empty
              v-if="problemRows.length === 0"
              description="没有问题单词，可以全部导入了"
              :image-size="80"
            />
            <TransitionGroup
              v-else
              name="problem"
              tag="div"
              class="problem-items"
            >
              <div
                v-for="r in problemRows"
                :key="r.index"
                class="problem-card"
                @click="locateRow(r.index)"
              >
                <div class="pc-row1">
                  <span class="pc-idx">#{{ r.index }}</span>
                  <el-tag
                    :type="tagType(r.status)"
                    effect="plain"
                  >
                    {{ r.problems[0]
                    }}<span v-if="r.problems.length > 1"
                      >+{{ r.problems.length - 1 }}</span
                    >
                  </el-tag>
                </div>
                <div class="pc-row2">
                  <span class="pc-label">单词：</span>
                  <span class="pc-value word-text">{{ r.key || "（空）" }}</span>
                </div>
                <div class="pc-row2">
                  <span class="pc-label">注音：</span>
                  <span class="pc-value phonetic-text">{{ r.value || "—" }}</span>
                </div>
                <div class="pc-row3">
                  <span class="pc-label">问题：</span>
                  <span class="pc-desc">{{
                    r.problems.join("；")
                  }}</span>
                </div>
                <div class="pc-actions">
                  <el-button
                    type="success"
                    plain
                    class="btn-cell gebinee--btn-success-hover"
                    @click.stop="locateRow(r.index)"
                  >
                    <el-icon><Edit /></el-icon>
                    <span>跳转编辑</span>
                  </el-button>
                  <el-button
                    type="danger"
                    plain
                    class="btn-cell"
                    @click.stop="discardRow(r.index)"
                  >
                    <el-icon><Delete /></el-icon>
                    <span>放弃导入</span>
                  </el-button>
                </div>
              </div>
            </TransitionGroup>
          </div>
          <div class="panel-footer">
            <GebineeButton
              type="warning"
              plain
              :disabled="problemRows.length === 0"
              @click="exportProblems"
            >
              <el-icon><Download /></el-icon>
              <span>导出问题项为 Excel</span>
            </GebineeButton>
          </div>
        </div>
      </div>
    </Transition>

    <div class="bottom-bar">
      <div class="bottom-stats">
        共 <b>{{ rows.length }}</b> 条 · 可导入
        <b class="num-success">{{ successCount }}</b> 条 · 问题
        <b class="num-danger">{{ problemRows.length }}</b> 条
      </div>
      <div class="bottom-actions">
        <GebineeButton type="default" @click="closeWindow">取消</GebineeButton>
        <GebineeButton
          class="gebinee--btn-green"
          :loading="importing"
          :disabled="successCount === 0"
          @click="importAll"
        >
          导入
        </GebineeButton>
      </div>
    </div>
  </div>
</template>

<style scoped>
.import-page {
  font-size: 16px;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  position: relative;
  background: var(--el-bg-color, #fff);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--el-border-color-lighter, #f2f6fc);
}
.toolbar .spacer {
  flex: 1;
}
.title-row {
  display: flex;
  align-items: baseline;
}
.panel-title {
  font-size: 18px;
  font-weight: 600;
}
.file-name {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  margin-left: 6px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.toolbar .count {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}
.switch-label {
  font-size: 14px;
  color: var(--el-text-color-regular);
}
.problem-toggle {
  font-size: 15px;
  padding: 8px 12px;
}

.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 8px 16px 0;
}
.pager {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 12px;
  padding: 8px 16px;
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

/* 表头 */
:deep(.el-table th.el-table__cell > .cell) {
  font-size: 16px;
}
:deep(.el-table__cell .cell) {
  font-size: 16px;
}
:deep(.el-tag) {
  font-size: 16px;
}

/* 分页器 jumper 字号 */
:deep(.el-pagination__jump) {
  font-size: 14px !important;
}
:deep(.el-pagination__jump .el-input__inner) {
  font-size: 14px !important;
}

.cell-text {
  display: inline-block;
  word-break: break-all;
  font-size: 18px;
}
.word-text {
  font-family: var(--gebinee-word-font), sans-serif;
}
.phonetic-text {
  font-family: var(--gebinee-phonetic-font), sans-serif;
}

.op-cell {
  display: flex;
  justify-content: center;
  gap: 8px;
}
:deep(.btn-cell) {
  font-size: 15px;
  padding: 8px 10px;
}

/* 行状态 */
:deep(.row-danger td.el-table__cell) {
  background: #fef0f0;
  border-color: #fab6b6;
}
:deep(.row-warning td.el-table__cell) {
  background: #fdf6ec;
  border-color: #f5dab1;
}
:deep(.row-ignored td.el-table__cell) {
  background: #f4f4f5;
  opacity: 0.55;
  border-color: #d4d4d4;
}
:deep(.row-highlight td.el-table__cell) {
  animation: highlight-flash 1.6s ease;
}
@keyframes highlight-flash {
  0% { background: #e6f4ff; }
  100% { background: transparent; }
}

/* 问题面板覆盖层 */
.problem-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 10;
  display: flex;
  justify-content: flex-end;
}
.problem-panel {
  width: 440px;
  background: var(--el-bg-color, #fff);
  display: flex;
  flex-direction: column;
  height: 100%;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.12);
}
.panel-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 12px 14px 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--el-border-color-light, #ebeef5);
}
.panel-header .panel-title {
  font-size: 18px;
  font-weight: 600;
}
.problem-count {
  font-size: 14px;
  color: var(--el-text-color-secondary);
}
.panel-close {
  margin-left: auto;
  font-size: 16px;
}

.stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  flex-shrink: 0;
  padding: 10px 14px;
}
.stat-item {
  border-radius: 8px;
  padding: 8px 4px;
  text-align: center;
  border: 1px solid transparent;
}
.stat-num {
  font-size: 20px;
  font-weight: 700;
  line-height: 1.2;
}
.stat-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin-top: 2px;
}
.stat-item.success { background: #f0f9eb; color: #67c23a; }
.stat-item.warning { background: #fdf6ec; color: #e6a23c; }
.stat-item.danger { background: #fef0f0; color: #f56c6c; }
.stat-item.ignored { background: #f4f4f5; color: #909399; }

.problem-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0 14px 10px;
}
.problem-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.problem-card {
  border: 1px solid var(--el-border-color-light, #ebeef5);
  border-radius: 8px;
  padding: 12px 14px;
  cursor: pointer;
  background: var(--el-bg-color, #fff);
  transition: box-shadow 0.18s, transform 0.18s;
}
.problem-card:hover {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  transform: translateY(-1px);
}
.pc-row1 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}
.pc-idx {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}
.pc-row2 {
  display: flex;
  align-items: baseline;
  gap: 6px;
  margin-bottom: 4px;
}
.pc-label {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
  min-width: 42px;
}
.pc-value {
  font-size: 18px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pc-row3 {
  display: flex;
  align-items: baseline;
  gap: 6px;
  margin-bottom: 8px;
}
.pc-desc {
  font-size: 16px;
  color: #e6a23c;
  line-height: 1.4;
}
.pc-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.pc-actions :deep(.btn-cell) {
  font-size: 15px;
  padding: 8px 10px;
}

.panel-footer {
  flex-shrink: 0;
  padding: 10px 14px;
  border-top: 1px solid var(--el-border-color-lighter, #f2f6fc);
  display: flex;
  justify-content: center;
}

/* 覆盖层动画 */
.overlay-enter-active,
.overlay-leave-active {
  transition: opacity 0.25s ease;
}
.overlay-enter-active .problem-panel,
.overlay-leave-active .problem-panel {
  transition: transform 0.25s ease;
}
.overlay-enter-from,
.overlay-leave-to {
  opacity: 0;
}
.overlay-enter-from .problem-panel {
  transform: translateX(100%);
}
.overlay-leave-to .problem-panel {
  transform: translateX(100%);
}

/* 问题卡片动画 */
.problem-enter-active,
.problem-leave-active {
  transition: all 0.32s ease;
}
.problem-enter-from {
  opacity: 0;
  transform: translateX(20px);
}
.problem-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
.problem-leave-active {
  position: absolute;
}

/* 底部 */
.bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 10px 16px;
  border-top: 1px solid var(--el-border-color-lighter, #f2f6fc);
}
.bottom-stats {
  font-size: 15px;
  color: var(--el-text-color-regular);
}
.bottom-stats .num-success { color: #67c23a; }
.bottom-stats .num-danger { color: #f56c6c; }
.bottom-actions {
  display: flex;
  gap: 10px;
}

/* 深色模式 */
html.dark .import-page { background: var(--el-bg-color-page, #141414); }
html.dark :deep(.row-danger td.el-table__cell) { background: #3a2424; border-color: #6b3a3a; }
html.dark :deep(.row-warning td.el-table__cell) { background: #3a3320; border-color: #6b562a; }
html.dark :deep(.row-ignored td.el-table__cell) { background: #2a2a2a; border-color: #3a3a3a; }
html.dark .stat-item.success { background: #1f3a1f; }
html.dark .stat-item.warning { background: #3a2f12; }
html.dark .stat-item.danger { background: #3a1f1f; }
html.dark .stat-item.ignored { background: #2a2a2a; }
html.dark .problem-card { background: var(--el-bg-color-overlay, #1d1e1f); }
html.dark .pc-desc { color: #e6a23c; }
html.dark .problem-overlay { background: rgba(0, 0, 0, 0.5); }
</style>