<template>
  <div class="search-section">
    <!-- Drag Region (Top Bar) -->
    <div class="drag-bar"></div>

    <div class="input-wrapper">
      <input
        ref="inputRef"
        :value="modelValue"
        @input="
          $emit('update:modelValue', ($event.target as HTMLInputElement).value)
        "
        type="text"
        class="search-input"
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
.search-section {
  position: relative;
  width: 100%;
  background: transparent;
}

.drag-bar {
  height: 24px;
  width: 100%;
  -webkit-app-region: drag; /* This makes it draggable */
  cursor: grab;
}

.drag-bar:active {
  cursor: grabbing;
}

.input-wrapper {
  padding: 0 24px 20px 24px;
}

.search-input {
  width: 100%;
  font-size: 20px;
  font-weight: 500;
  color: var(--text-main);
  background: transparent;
  border: none;
  outline: none;
  -webkit-app-region: no-drag; /* Input must be non-draggable to be clickable */
}

.search-input::placeholder {
  color: var(--text-subtle);
}
</style>
