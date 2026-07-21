<script setup>
import { ref } from "vue";
import AddPanel from "../components/AddPanel.vue";
import QueryPanel from "../components/QueryPanel.vue";
import WordTable from "../components/WordTable.vue";

const emit = defineEmits(["changed"]);

const wordTableRef = ref(null);

function onChanged() {
  emit("changed");
  // 添加/编辑/删除后刷新右侧表格
  wordTableRef.value?.refresh();
}

// 供外部调用：刷新右侧全部单词预览区域
function refreshWordTable() {
  wordTableRef.value?.refresh();
}

defineExpose({ refreshWordTable });
</script>

<template>
  <div class="main-view">
    <aside class="left-pane">
      <section class="pane-section add-section">
        <AddPanel @changed="onChanged" />
      </section>
      <section class="pane-section query-section">
        <QueryPanel @changed="onChanged" />
      </section>
    </aside>
    <section class="right-pane">
      <WordTable ref="wordTableRef" @changed="onChanged" />
    </section>
  </div>
</template>

<style scoped>
.main-view {
  display: flex;
  height: 100%;
  gap: 12px;
  min-height: 0;
}
.left-pane {
  width: 340px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 20px;
  min-height: 0;
  overflow-y: auto;
}
.pane-section {
  background: var(--el-bg-color, #fff);
  border: 1px solid var(--el-border-color-light, #ebeef5);
  border-radius: 6px;
  padding: 12px 14px;
}
.add-section {
  flex-shrink: 0;
}
.query-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.right-pane {
  flex: 1;
  min-width: 0;
  background: var(--el-bg-color, #fff);
  border: 1px solid var(--el-border-color-light, #ebeef5);
  border-radius: 6px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
</style>
