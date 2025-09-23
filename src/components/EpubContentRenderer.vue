<template>
  <div>
    <!-- 渲染模式切换 -->
    <div class="render-mode-switch">
      <n-radio-group
        v-model:value="renderMode"
        size="small"
      >
        <n-radio value="html">富文本模式</n-radio>
        <n-radio value="text">纯文本模式</n-radio>
      </n-radio-group>
    </div>

    <!-- HTML渲染模式 -->
    <div
      v-if="renderMode === 'html' && currentChapter"
      class="chapter-content-html"
      v-html="sanitizedHtmlContent"
    ></div>

    <!-- 纯文本模式 -->
    <div
      v-else-if="renderMode === 'text' && currentChapter"
      class="chapter-content"
    >
      {{ currentChapter.content }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { NRadioGroup, NRadio } from "naive-ui";
import DOMPurify from "dompurify";
import { type Chapter, RenderMode } from "../services/fileService";

interface Props {
  currentChapter: Chapter | null;
  chapters: Chapter[];
  currentChapterIndex: number;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  "chapter-changed": [index: number];
}>();

const renderMode = ref<RenderMode>(RenderMode.HTML);

const sanitizedHtmlContent = computed(() => {
  if (!props.currentChapter?.html_content) return "";

  // 使用DOMPurify清理HTML，但保留图片
  return DOMPurify.sanitize(props.currentChapter.html_content, {
    ALLOWED_TAGS: ["p", "div", "span", "h1", "h2", "h3", "h4", "h5", "h6", "br", "img", "strong", "em", "u", "i", "b"],
    ALLOWED_ATTR: ["src", "alt", "title", "class", "style"],
    ALLOW_DATA_ATTR: false,
  });
});
</script>

<style scoped>
.render-mode-switch {
  padding: 8px 16px;
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-card-color);
}

.chapter-content-html {
  font-size: 16px;
  line-height: 1.8;
  color: var(--n-text-color);
}

.chapter-content-html :deep(img) {
  width: 100% !important;
  height: 100% !important;
  max-width: 100%;
  height: auto;
  display: block;
  margin: 16px auto;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.chapter-content-html :deep(p) {
  margin-bottom: 16px;
  text-indent: 2em;
}

.chapter-content-html :deep(h1),
.chapter-content-html :deep(h2),
.chapter-content-html :deep(h3) {
  margin: 24px 0 16px 0;
  font-weight: 600;
}

.chapter-content-html :deep(.epub-footnote) {
  width: 1rem !important;
  height: 1rem !important;
  display: inline-block;
  font-size: 0.85em;
  vertical-align: super;
  margin: 0 2px;
  padding: 0 5px;
  border-radius: 4px;
  background-color: rgba(var(--n-primary-color-rgb), 0.1);
  color: var(--n-primary-color);
  text-decoration: none;
  line-height: 1.2;
  transition: all 0.2s ease;

  &:hover {
    background-color: rgba(var(--n-primary-color-rgb), 0.2);
    transform: translateY(-1px);
  }
}

.chapter-content {
  font-size: 16px;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: var(--n-text-color);
}

.chapter-navigation {
  display: flex;
  justify-content: center;
  padding: 16px 0;
}

.chapter-navigation.top {
  margin-bottom: 16px;
}

.chapter-navigation.bottom {
  margin-top: 24px;
}
</style>
