<template>
  <div>
    <div
      class="chapter-title"
      v-if="currentChapter"
    >
      {{ currentChapter.title }}
    </div>
    <div
      class="chapter-content"
      v-if="currentChapter"
    >
      {{ currentChapter.content }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { type Chapter } from "../services/fileService";

interface Props {
  currentChapter: Chapter | null;
}

defineProps<Props>();
</script>

<style scoped>
.chapter-title {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 24px;
  text-align: center;
  color: var(--n-text-color);
}

.chapter-content {
  font-size: 16px;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: var(--n-text-color);
}
</style>