<template>
  <div id="app-wrapper" @click.self="handleGlobalClick">
    <div class="spotlight-container" @click.stop>
      <SearchBar
        v-model="query"
        @search="handleSearch"
        @escape="handleEscape"
      />

      <ResultCard
        :loading="loading"
        :error="error"
        :bibtex="formattedBibtex"
        :inputType="inputType"
        :copied="copied"
        @copy="copyBibtex"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, Ref } from "vue";
import SearchBar from "./components/SearchBar.vue";
import ResultCard from "./components/ResultCard.vue";
import { formatBibtex } from "./utils/bibtex";

const API_BASE = "http://127.0.0.1:8765";

interface ResolveResponse {
  success: boolean;
  type: string;
  bibtex: string | null;
  error: string | null;
}

const query: Ref<string> = ref("");
const rawBibtex: Ref<string> = ref("");
const inputType: Ref<string> = ref("");
const loading: Ref<boolean> = ref(false);
const error: Ref<string> = ref("");
const copied: Ref<boolean> = ref(false);

const formattedBibtex = computed(() => formatBibtex(rawBibtex.value));

function handleGlobalClick() {
  handleEscape();
}

function handleEscape() {
  if (window.electronAPI?.hideWindow) {
    window.electronAPI.hideWindow();
  }
}

// Global key handler
function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    handleEscape();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});

async function handleSearch() {
  const q = query.value.trim();
  if (!q) return;

  loading.value = true;
  error.value = "";
  rawBibtex.value = "";
  copied.value = false;

  try {
    const response = await fetch(
      `${API_BASE}/resolve?q=${encodeURIComponent(q)}`,
    );
    const data: ResolveResponse = await response.json();

    if (data.success && data.bibtex) {
      rawBibtex.value = data.bibtex;
      inputType.value = data.type;
    } else {
      error.value = data.error || "Failed to resolve. Please check your input.";
    }
  } catch (err) {
    error.value =
      "Cannot connect to backend. Please ensure Python server is running.";
    console.error(err);
  } finally {
    loading.value = false;
  }
}

async function copyBibtex() {
  if (!formattedBibtex.value) return;

  try {
    // Copy the FORMATTED bibtex to be consistent with what the user sees
    if (window.electronAPI?.copyToClipboard) {
      await window.electronAPI.copyToClipboard(formattedBibtex.value);
    } else {
      await navigator.clipboard.writeText(formattedBibtex.value);
    }
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (err) {
    console.error("Copy failed:", err);
  }
}
</script>

<style>
/* Global styles wrapper */
#app-wrapper {
  width: 100vw;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
