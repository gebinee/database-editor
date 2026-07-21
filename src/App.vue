<script setup>
import { ref, onMounted } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import zhCn from "element-plus/es/locale/lang/zh-cn";
import { useSettingsStore } from "./stores/settings";
import { pickDatabaseFile, switchDatabase } from "./api/settings";
import { errorMessage } from "./utils/error";
import MainView from "./views/MainView.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import { EditPen, Setting } from "@element-plus/icons-vue";

const settingsStore = useSettingsStore();

const settingsDialogVisible = ref(false);
const mainViewRef = ref(null);

// Element Plus 中文本地化（自定义 pagination goto 文案）
const customLocale = {
  ...zhCn,
  el: {
    ...zhCn.el,
    pagination: {
      ...zhCn.el.pagination,
      goto: "跳转到",
    },
  },
};

onMounted(async () => {
  try {
    await settingsStore.init();
    if (settingsStore.dbMissing) {
      // 非首次启动但数据库文件不存在，提示用户选择处理方式
      await promptMissingDb();
    } else if (settingsStore.dbError) {
      ElMessage.warning(
        `数据库打开失败：${settingsStore.dbError}。请在设置中配置有效的数据库路径。`
      );
    }
  } catch (e) {
    ElMessage.error(`初始化失败：${e?.message || e}`);
  }
});

// 数据库文件不存在时的处理：让用户选择已有文件或创建新库
async function promptMissingDb() {
  const path = settingsStore.settings?.db_path || "";
  try {
    await ElMessageBox.confirm(
      `数据库文件 "${path}" 不存在，可能是路径错误或文件已被移动。\n请选择已有的数据库文件，或创建一个新的空数据库。`,
      "数据库文件不存在",
      {
        confirmButtonText: "创建新库",
        cancelButtonText: "选择文件",
        type: "warning",
        distinguishCancelAndClose: true,
        closeOnClickModal: false,
      }
    );
    // 用户点击"创建新库"：在原路径创建空库
    await switchDatabase(path);
    settingsStore.clearDbMissing();
    ElMessage.success("已创建新数据库");
  } catch (action) {
    if (action === "cancel") {
      // 用户点击"选择文件"
      try {
        const filePath = await pickDatabaseFile();
        if (filePath) {
          await switchDatabase(filePath);
          settingsStore.clearDbMissing();
          ElMessage.success("数据库已切换");
        } else {
          ElMessage.warning("未选择数据库文件，请在设置中配置");
        }
      } catch (e) {
        ElMessage.error(`切换数据库失败：${errorMessage(e)}`);
      }
    } else {
      // 用户关闭对话框
      ElMessage.warning("请在设置中配置有效的数据库路径");
    }
  }
}

function openSettings() {
  settingsDialogVisible.value = true;
}

// 设置保存后刷新右侧全部单词预览区域
function onSettingsChanged() {
  mainViewRef.value?.refreshWordTable();
}
</script>

<template>
  <el-config-provider :locale="customLocale">
    <div class="app-layout">
      <header class="app-header">
        <div class="title">
          <el-icon :size="22" color="#409eff"><EditPen /></el-icon>
          <span class="title-text">图形化数据库编辑工具</span>
        </div>
        <div class="actions">
          <el-tooltip content="软件设置" placement="bottom">
            <el-button circle @click="openSettings">
              <el-icon><Setting /></el-icon>
            </el-button>
          </el-tooltip>
        </div>
      </header>

      <main class="app-main">
        <el-skeleton v-if="!settingsStore.ready" :rows="6" animated />
        <MainView v-else ref="mainViewRef" />
      </main>

      <SettingsDialog
        v-model:visible="settingsDialogVisible"
        @changed="onSettingsChanged"
      />
    </div>
  </el-config-provider>
</template>

<style scoped>
.actions {
  display: flex;
  gap: 8px;
}
.title-text {
  color: #409eff;
}
</style>
