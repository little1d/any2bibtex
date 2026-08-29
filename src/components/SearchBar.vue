<template>
  <form class="search-form" @submit.prevent="emit('search')">
    <Search :size="19" :stroke-width="1.8" aria-hidden="true" />
    <input
      ref="inputRef"
      :value="modelValue"
      type="text"
      class="search-input"
      placeholder="DOI, arXiv ID, or paper title"
      autocomplete="off"
      spellcheck="false"
      autofocus
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keydown.esc="emit('escape')"
    />
    <span class="input-type" :class="{ invisible: !modelValue.trim() }">{{ inputType }}</span>
    <button
      class="submit-button"
      type="submit"
      :disabled="!modelValue.trim() || loading"
      aria-label="Resolve paper"
      title="Resolve paper"
    >
      <LoaderCircle
        v-if="loading"
        class="spin"
        :size="17"
        :stroke-width="1.9"
        aria-hidden="true"
      />
      <ArrowRight v-else :size="17" :stroke-width="1.9" aria-hidden="true" />
    </button>
  </form>
</template>

<script setup lang="ts">
import { ArrowRight, LoaderCircle, Search } from "@lucide/vue";
import { onMounted, ref } from "vue";

defineProps<{
  modelValue: string;
  inputType: string;
  loading: boolean;
}>();

const emit = defineEmits<{
  (event: "update:modelValue", value: string): void;
  (event: "search"): void;
  (event: "escape"): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);

onMounted(() => {
  inputRef.value?.focus();
});

defineExpose({
  focus: () => inputRef.value?.focus(),
});
</script>

<style scoped>
.search-form {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr) auto 32px;
  align-items: center;
  gap: 10px;
  min-height: 68px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-soft);
  color: var(--text-subtle);
  background: var(--surface-bg);
}

.search-input {
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-main);
  font-size: 17px;
  font-weight: 450;
  line-height: 1.4;
}

.search-input::placeholder {
  color: var(--text-subtle);
}

.input-type {
  padding: 3px 6px;
  border-radius: 4px;
  background: var(--surface-raised);
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 650;
  text-transform: uppercase;
}

.input-type.invisible {
  visibility: hidden;
}

.submit-button {
  display: grid;
  width: 32px;
  height: 32px;
  place-items: center;
  border: 1px solid var(--border-strong);
  border-radius: 7px;
  background: var(--surface-raised);
  color: var(--text-main);
  cursor: pointer;
  transition:
    border-color 140ms ease,
    background-color 140ms ease,
    color 140ms ease,
    transform 140ms ease;
}

.submit-button:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--accent) 45%, var(--border-strong));
  background: var(--accent-soft);
  color: var(--accent);
}

.submit-button:active:not(:disabled) {
  transform: scale(0.96);
}

.submit-button:disabled {
  cursor: default;
  opacity: 0.38;
}

.spin {
  animation: spin 800ms linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
