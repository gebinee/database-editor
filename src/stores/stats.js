import { defineStore } from "pinia";
import { ref } from "vue";
import { getStats } from "../api/db";

export const useStatsStore = defineStore("stats", () => {
  const totalCount = ref(0);
  const ready = ref(false);

  async function refresh() {
    try {
      const s = await getStats();
      totalCount.value = s.total_count;
    } catch (e) {
      // 数据库未打开等情况静默处理
      totalCount.value = 0;
    } finally {
      ready.value = true;
    }
  }

  return { totalCount, ready, refresh };
});
