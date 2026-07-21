import { defineStore } from "pinia";
import { ref } from "vue";

export const useQueryStore = defineStore("query", () => {
  const search = ref("");
  const page = ref(1);
  const pageSize = ref(20);
  const sortOrder = ref("asc"); // asc | desc

  function resetToFirstPage() {
    page.value = 1;
  }

  return { search, page, pageSize, sortOrder, resetToFirstPage };
});
