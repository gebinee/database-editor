<script setup>
import { reactive, ref, watch } from "vue";
import { Close, Check } from "@element-plus/icons-vue";
import { updateEntry, updateEntryKey } from "../api/db";
import {
  checkEmptyText,
  checkKeyValidity,
  emptyMsg,
  invalidKeyMsg,
} from "../utils/validation";
import { errorMessage } from "../utils/error";
import { GebineeButton, GebineeInput } from "@gebinee/components";

const props = defineProps({
  visible: { type: Boolean, default: false },
  entry: { type: Object, default: null }, // { key, value }
});
const emit = defineEmits(["update:visible", "changed"]);

const form = reactive({ key: "", value: "" });
const originalKey = ref("");
const saving = ref(false);

watch(
  () => props.visible,
  (v) => {
    if (v && props.entry) {
      form.key = props.entry.key;
      form.value = props.entry.value;
      originalKey.value = props.entry.key;
    }
  }
);

async function onSave() {
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
  saving.value = true;
  try {
    if (key !== originalKey.value) {
      await updateEntryKey(originalKey.value, key, value);
    } else {
      await updateEntry(key, value);
    }
    ElMessage.success("保存成功");
    emit("changed");
    emit("update:visible", false);
  } catch (e) {
    ElMessage.error(errorMessage(e));
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <el-dialog
    :model-value="visible"
    @update:model-value="emit('update:visible', $event)"
    width="460px"
    align-center
    destroy-on-close
    class="edit-dialog"
  >
    <template #header>
      <div class="dialog-header-spacer"></div>
    </template>
    <el-form :model="form" label-position="left" label-width="80px">
      <el-form-item label="单词" class="compact-item">
        <GebineeInput
          v-model="form.key"
          placeholder="单词（仅英文字母和连字符）"
          class="word-input"
          clearable
        />
      </el-form-item>
      <el-form-item label="注音结果" class="compact-item">
        <GebineeInput
          v-model="form.value"
          placeholder="注音结果"
          class="phonetic-input"
          clearable
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <GebineeButton @click="emit('update:visible', false)">
        <el-icon><Close /></el-icon>
        <span>取消</span>
      </GebineeButton>
      <GebineeButton class="gebinee--btn-green" :loading="saving" @click="onSave">
        <el-icon><Check /></el-icon>
        <span>保存</span>
      </GebineeButton>
    </template>
  </el-dialog>
</template>

<style scoped>
.dialog-header-spacer {
  height: 20px;  /* 根据需要的间距调整 */
}
:deep(.compact-item) {
  margin-bottom: 14px;
}

/*noinspection CssUnusedSymbol*/
:deep(.compact-item .el-form-item__label) {
  font-size: 16px;
  line-height: 40px;
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
