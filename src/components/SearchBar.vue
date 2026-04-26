<template>
  <div class="relative w-full bg-transparent">
    <!-- Drag Region (Top Bar) -->
    <div class="drag-bar h-6 w-full cursor-grab active:cursor-grabbing"></div>

    <div class="px-6 pb-5">
      <input
        ref="inputRef"
        :value="modelValue"
        @input="
          $emit('update:modelValue', ($event.target as HTMLInputElement).value)
        "
        type="text"
        class="search-input w-full border-0 bg-transparent text-xl font-medium text-white outline-none placeholder:text-blue-300/50"
        placeholder="Enter DOI, arXiv ID, or paper title..."
        @keyup.enter="$emit('search')"
        @keydown.esc="handleEsc"
        autofocus
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";

const props = defineProps<{
  modelValue: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "search"): void;
  (e: "escape"): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);

function handleEsc() {
  emit("escape");
}

onMounted(() => {
  inputRef.value?.focus();
});

defineExpose({
  focus: () => inputRef.value?.focus(),
});
</script>

<style scoped>
.drag-bar {
  -webkit-app-region: drag;
}

.search-input {
  -webkit-app-region: no-drag;
}
</style>
