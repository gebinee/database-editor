<script setup>
import { ref, computed, watch, nextTick } from "vue";
import {
  readExcelForImport,
  checkDuplicates,
  importEntries,
  exportProblemWords,
} from "../api/db";
import { pickSavePath } from "../api/settings";
import { checkEmptyText, checkKeyValidity } from "../utils/validation";
import { errorMessage } from "../utils/error";
import {Download} from "@element-plus/icons-vue";

const props = defineProps({
  visible: { type: Boolean, default: false },
  filePath: { type: String, default: "" },
});
const emit = defineEmits(["update:visible", "imported"]);

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit("update:visible", v),
});

const PAGE_SIZE = 100;
const loading = ref(false);
const importing = ref(false);
const rows = ref([]);
const fileName = ref("");
const dbExistsCache = ref(new Map());
const highlightIndex = ref(null);
const onlyProblems = ref(false);
const page = ref(1);
const tableRef = ref();

watch(
  () => props.visible,
  async (v) => {
    if (v && props.filePath) {
      await load();
    } else if (!v) {
      reset();
    }
  }
);

function reset() {
  rows.value = [];
  fileName.value = "";
  dbExistsCache.value = new Map();
  highlightIndex.value = null;
  onlyProblems.value = false;
  page.value = 1;
}

async function load() {
  loading.value = true;
  try {
    fileName.value = props.filePath.split(/[\\/]/).pop() || props.filePath;
    const raw = await readExcelForImport(props.filePath);
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
    // DB 重复预判：一次性批量查询所有非空合法 key
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
    emit("update:visible", false);
  } finally {
    loading.value = false;
  }
}

// 本地校验 + Excel 内重复（不动 DB已存在 与 ignored 状态）
function recomputeLocal() {
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

// 应用 DB已存在 + 计算 status（ignored 行保持 ignored）
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
  const start = (page.value - 1) * PAGE_SIZE;
  return filteredRows.value.slice(start, start + PAGE_SIZE);
});
const totalFiltered = computed(() => filteredRows.value.length);

// 行内编辑
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
      // 查询失败：保守起见不更新缓存，保持原值
    }
  }
  recomputeStatus();
}

// 定位高亮 + 进入编辑
async function locateRow(index) {
  let pos = filteredRows.value.findIndex((r) => r.index === index);
  if (pos < 0) {
    // 可能被"只看问题项"过滤，先关闭过滤
    onlyProblems.value = false;
    await nextTick();
    pos = filteredRows.value.findIndex((r) => r.index === index);
    if (pos < 0) return;
  }
  page.value = Math.floor(pos / PAGE_SIZE) + 1;
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
  r.status = "danger"; // 临时标记，使 recomputeStatus 重新计算该行
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
    emit("imported");
    emit("update:visible", false);
  } catch (e) {
    ElMessage.error(errorMessage(e));
  } finally {
    importing.value = false;
  }
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
  <el-dialog
    v-model="dialogVisible"
    :title="`从 Excel 导入${fileName ? ' · ' + fileName : ''}`"
    width="92vw"
    top="4vh"
    align-center
    destroy-on-close
    :close-on-click-modal="false"
    class="import-dialog"
  >
    <div v-loading="loading" class="import-body">
      <el-row :gutter="16" class="import-row">
        <!-- 左栏：数据表 -->
        <el-col :span="15">
          <div class="left-panel">
            <div class="toolbar">
              <span class="panel-title">数据预览</span>
              <span class="count">共 {{ rows.length }} 行</span>
              <div class="spacer" />
              <span class="switch-label">只看问题项</span>
              <el-switch v-model="onlyProblems" size="small" />
            </div>
            <div class="table-wrap">
              <el-table
                ref="tableRef"
                :data="pagedRows"
                height="100%"
                :row-class-name="rowClassName"
                @row-dblclick="startEdit"
                border
                size="small"
                empty-text="暂无数据"
              >
                <el-table-column
                  label="#"
                  prop="index"
                  width="60"
                  align="center"
                />
                <el-table-column label="单词" min-width="170">
                  <template #default="{ row }">
                    <el-input
                      v-if="row.editing"
                      v-model="row.draftKey"
                      size="small"
                      placeholder="单词"
                      @keyup.enter="saveEdit(row)"
                      @keyup.esc="cancelEdit(row)"
                    />
                    <span v-else class="font-word cell-text">{{
                      row.key || "—"
                    }}</span>
                  </template>
                </el-table-column>
                <el-table-column label="注音" min-width="190">
                  <template #default="{ row }">
                    <el-input
                      v-if="row.editing"
                      v-model="row.draftValue"
                      size="small"
                      placeholder="注音"
                      @keyup.enter="saveEdit(row)"
                      @keyup.esc="cancelEdit(row)"
                    />
                    <span v-else class="font-phonetic cell-text">{{
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
                      size="small"
                      effect="light"
                    >
                      {{ tagText(row.status) }}
                    </el-tag>
                  </template>
                </el-table-column>
                <el-table-column
                  label="操作"
                  width="150"
                  align="center"
                  fixed="right"
                >
                  <template #default="{ row }">
                    <template v-if="row.editing">
                      <el-button
                        link
                        type="primary"
                        size="small"
                        @click="saveEdit(row)"
                        >保存</el-button
                      >
                      <el-button link size="small" @click="cancelEdit(row)"
                        >取消</el-button
                      >
                    </template>
                    <template v-else>
                      <el-button
                        link
                        type="primary"
                        size="small"
                        @click="startEdit(row)"
                        >编辑</el-button
                      >
                      <el-button
                        v-if="row.status !== 'ignored'"
                        link
                        type="danger"
                        size="small"
                        @click="discardRow(row.index)"
                        >放弃</el-button
                      >
                      <el-button
                        v-else
                        link
                        type="success"
                        size="small"
                        @click="restoreRow(row.index)"
                        >恢复</el-button
                      >
                    </template>
                  </template>
                </el-table-column>
              </el-table>
            </div>
            <div class="pager">
              <el-pagination
                v-model:current-page="page"
                :page-size="PAGE_SIZE"
                :total="totalFiltered"
                layout="prev, pager, next, total"
                small
                background
              />
            </div>
          </div>
        </el-col>

        <!-- 右栏：问题面板 -->
        <el-col :span="9">
          <div class="right-panel">
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

            <div class="problem-header">
              <span class="panel-title"
                >问题单词（{{ problemRows.length }}）</span
              >
              <span class="hint">点击定位 · 修复后自动移除</span>
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
                    <span class="pc-word font-word">{{ r.key || "（空）" }}</span>
                    <el-tag
                      :type="tagType(r.status)"
                      size="small"
                      effect="plain"
                    >
                      {{ r.problems[0]
                      }}<span v-if="r.problems.length > 1"
                        >+{{ r.problems.length - 1 }}</span
                      >
                    </el-tag>
                  </div>
                  <div class="pc-row2 font-phonetic">{{ r.value || "—" }}</div>
                  <div class="pc-actions">
                    <el-button
                      size="small"
                      type="primary"
                      @click.stop="locateRow(r.index)"
                      >跳转编辑</el-button
                    >
                    <el-button
                      size="small"
                      link
                      type="danger"
                      @click.stop="discardRow(r.index)"
                      >放弃导入</el-button
                    >
                  </div>
                </div>
              </TransitionGroup>
            </div>

            <div class="right-footer">
              <el-button
                :disabled="problemRows.length === 0"
                @click="exportProblems"
              >
                <el-icon><Download /></el-icon>
                <span>导出问题项为 Excel</span>
              </el-button>
            </div>
          </div>
        </el-col>
      </el-row>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <div class="footer-stats">
          共 <b>{{ rows.length }}</b> 条 · 可导入
          <b class="num-success">{{ successCount }}</b> 条 · 问题
          <b class="num-danger">{{ problemRows.length }}</b> 条
        </div>
        <div class="footer-actions">
          <el-button @click="dialogVisible = false">取消</el-button>
          <el-button
            type="primary"
            :loading="importing"
            :disabled="successCount === 0"
            @click="importAll"
          >
            全部导入可用项（{{ successCount }}）
          </el-button>
        </div>
      </div>
    </template>
  </el-dialog>
</template>

<style scoped>
.import-body {
  height: calc(82vh - 20px);
}
.import-row {
  height: 100%;
}
.left-panel,
.right-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 2px 8px;
  flex-shrink: 0;
}
.toolbar .spacer {
  flex: 1;
}
.panel-title {
  font-weight: 600;
  font-size: 14px;
}
.toolbar .count {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.switch-label {
  font-size: 12px;
  color: var(--el-text-color-regular);
}

.table-wrap {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.pager {
  flex-shrink: 0;
  display: flex;
  justify-content: flex-end;
  padding-top: 8px;
}

.cell-text {
  display: inline-block;
  word-break: break-all;
}

/* 行状态背景 */
/*noinspection CssUnusedSymbol*/
:deep(.row-danger td.el-table__cell) {
  background: #fef0f0;
}

/*noinspection CssUnusedSymbol*/
:deep(.row-warning td.el-table__cell) {
  background: #fdf6ec;
}

/*noinspection CssUnusedSymbol*/
:deep(.row-ignored td.el-table__cell) {
  background: #f4f4f5;
  opacity: 0.55;
}
:deep(.row-highlight td.el-table__cell) {
  animation: highlight-flash 1.6s ease;
}
@keyframes highlight-flash {
  0% {
    background: #e6f4ff;
  }
  100% {
    background: transparent;
  }
}

/* 右栏统计卡片 */
.stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  flex-shrink: 0;
  margin-bottom: 10px;
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
  font-size: 11px;
  color: var(--el-text-color-secondary);
  margin-top: 2px;
}
.stat-item.success {
  background: #f0f9eb;
  color: #67c23a;
}
.stat-item.warning {
  background: #fdf6ec;
  color: #e6a23c;
}
.stat-item.danger {
  background: #fef0f0;
  color: #f56c6c;
}
.stat-item.ignored {
  background: #f4f4f5;
  color: #909399;
}

.problem-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  flex-shrink: 0;
  padding: 4px 2px 8px;
}
.problem-header .hint {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.problem-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding-right: 2px;
}
.problem-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.problem-card {
  border: 1px solid var(--el-border-color-light, #ebeef5);
  border-radius: 8px;
  padding: 8px 10px;
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
}
.pc-idx {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  flex-shrink: 0;
}
.pc-word {
  font-weight: 600;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pc-row2 {
  font-size: 13px;
  color: var(--el-text-color-secondary);
  margin: 4px 0 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pc-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.right-footer {
  flex-shrink: 0;
  padding-top: 10px;
  border-top: 1px solid var(--el-border-color-lighter, #f2f6fc);
  display: flex;
  justify-content: center;
}

/* 进入/离开动画 */
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
  width: calc(100% - 2px);
}

/* 底部 */
.dialog-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.footer-stats {
  font-size: 13px;
  color: var(--el-text-color-regular);
}
.footer-stats .num-success {
  color: #67c23a;
}
.footer-stats .num-danger {
  color: #f56c6c;
}
.footer-actions {
  display: flex;
  gap: 10px;
}

/* 深色模式 */
html.dark :deep(.row-danger td.el-table__cell) {
  background: #3a2424;
}
html.dark :deep(.row-warning td.el-table__cell) {
  background: #3a3320;
}
html.dark :deep(.row-ignored td.el-table__cell) {
  background: #2a2a2a;
}
html.dark .stat-item.success {
  background: #1f3a1f;
}
html.dark .stat-item.warning {
  background: #3a2f12;
}
html.dark .stat-item.danger {
  background: #3a1f1f;
}
html.dark .stat-item.ignored {
  background: #2a2a2a;
}
html.dark .problem-card {
  background: var(--el-bg-color-overlay, #1d1e1f);
}
</style>
