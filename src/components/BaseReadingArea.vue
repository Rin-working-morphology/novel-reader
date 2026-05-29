<template>
  <n-layout-content>
    <div class="reading-area">
      <div
        v-if="!currentFile"
        class="empty-state"
      >
        <n-empty description="请选择一个文件开始阅读">
          <template #icon>
            <n-icon size="48">
              <BookOutline />
            </n-icon>
          </template>
        </n-empty>
      </div>

      <!-- 添加loading状态 -->
      <div
        v-else-if="loading"
        class="loading-state"
      >
        <n-spin size="large">
          <template #description>
            <div class="loading-text">小祥●▛▙小祥●▛▙小祥●▛▙...</div>
          </template>
        </n-spin>
      </div>

      <div
        v-else
        class="content-area"
      >
        <!-- 文章内容 -->
        <n-scrollbar
          ref="contentRef"
          :content-class="`content-text ${theme === 'default' ? 'frosted-glass' : ''}`"
          @scroll="handleScroll"
        >
          <div class="reading-flow">
            <div
              v-if="hasPreviousChapter"
              class="chapter-boundary chapter-boundary--previous"
            >
              <button
                type="button"
                class="chapter-boundary-button"
                @click="handleJumpToChapter(currentChapterIndex - 1)"
              >
                <n-icon size="20">
                  <ArrowBackOutline />
                </n-icon>
                <span class="chapter-boundary-copy">
                  <span class="chapter-boundary-action">返回上一章</span>
                  <span class="chapter-boundary-title">{{ chapters[currentChapterIndex - 1]?.title }}</span>
                </span>
              </button>
            </div>

            <div
              ref="chapterContentRef"
              class="chapter-content-anchor"
            >
              <!-- 内容渲染插槽 -->
              <slot
                name="content"
                :current-chapter="currentChapter"
                :chapters="chapters"
                :current-chapter-index="currentChapterIndex"
              />
            </div>

            <div
              v-if="hasNextChapter"
              class="chapter-boundary chapter-boundary--next"
            >
              <button
                type="button"
                class="chapter-boundary-button"
                @click="handleJumpToChapter(currentChapterIndex + 1)"
              >
                <span class="chapter-boundary-copy">
                  <span class="chapter-boundary-action">前往下一章</span>
                  <span class="chapter-boundary-title">{{ chapters[currentChapterIndex + 1]?.title }}</span>
                </span>
                <n-icon size="20">
                  <ArrowForwardOutline />
                </n-icon>
              </button>
            </div>
          </div>
        </n-scrollbar>
      </div>
    </div>
  </n-layout-content>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { NLayoutContent, NEmpty, NIcon, NScrollbar, NSpin } from "naive-ui";
import { BookOutline, ArrowForwardOutline, ArrowBackOutline } from "@vicons/ionicons5";
import { type TxtFile, type Chapter } from "../services/fileService";

interface Props {
  currentFile: TxtFile | null;
  chapters: Chapter[];
  currentChapterIndex: number;
  theme: string;
  loading: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "chapter-changed": [index: number];
  scroll: [position: number];
}>();

const contentRef = ref();
const chapterContentRef = ref<HTMLElement | null>(null);

const currentChapter = computed(() => {
  return props.chapters[props.currentChapterIndex] || null;
});

const hasPreviousChapter = computed(() => {
  return props.chapters.length > 1 && props.currentChapterIndex > 0;
});

const hasNextChapter = computed(() => {
  return props.chapters.length > 1 && props.currentChapterIndex < props.chapters.length - 1;
});

const checkContentHeight = () => {};

const handleJumpToChapter = (index: number) => {
  emit("chapter-changed", index);
};

const handleScroll = () => {
  if (contentRef.value?.scrollbarInstRef) {
    const scrollTop = contentRef.value.scrollbarInstRef.containerScrollTop;
    emit("scroll", scrollTop);
  }
};

const getChapterContentScrollTop = () => {
  const scrollbar = contentRef.value?.scrollbarInstRef;
  const container = scrollbar?.containerRef;
  const chapterContent = chapterContentRef.value;

  if (!scrollbar || !container || !chapterContent) {
    return 0;
  }

  return scrollbar.containerScrollTop + chapterContent.getBoundingClientRect().top - container.getBoundingClientRect().top;
};

// 暴露方法供父组件调用
const scrollToPosition = (position: number) => {
  if (contentRef.value) {
    const top = position === 0 && hasPreviousChapter.value ? getChapterContentScrollTop() : position;
    contentRef.value.scrollTo({ top });
  }
};

defineExpose({
  contentRef,
  scrollToPosition,
  checkContentHeight,
});
</script>

<style scoped>
.reading-area {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.content-area {
  height: 100%;
  display: flex;
  flex-direction: column;
}

:deep(.n-scrollbar-content.content-text) {
  flex: 1;
  overflow-y: auto;
  padding: 96px 24px 112px;
  line-height: 1.8;
}

.reading-flow {
  width: min(100%, 72ch);
  margin: 0 auto;
}

.frosted-glass {
  background-color: rgba(var(--n-color), 0.1);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 8px;
}

.chapter-boundary {
  display: flex;
  justify-content: center;
  color: var(--n-text-color);
}

.chapter-boundary--previous {
  margin-bottom: 48px;
}

.chapter-boundary--next {
  margin-top: 64px;
}

.chapter-boundary-button {
  width: min(100%, 420px);
  min-height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  padding: 12px 18px;
  border: 1px solid var(--n-border-color);
  border-radius: 8px;
  color: inherit;
  background: color-mix(in srgb, var(--n-card-color) 78%, transparent);
  cursor: pointer;
  font: inherit;
  line-height: 1.4;
  text-align: left;
  transition:
    background-color 0.2s ease,
    border-color 0.2s ease,
    color 0.2s ease;
}

.chapter-boundary-button:hover {
  background: var(--n-hover-color);
  border-color: color-mix(in srgb, var(--n-primary-color) 45%, var(--n-border-color));
  color: var(--n-primary-color);
}

.chapter-boundary-button:focus-visible {
  outline: 2px solid var(--n-primary-color);
  outline-offset: 3px;
}

.chapter-boundary-copy {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.chapter-boundary-action {
  font-size: 14px;
  font-weight: 600;
}

.chapter-boundary-title {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  opacity: 0.78;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  flex-direction: column;
}

.loading-text {
  margin-top: 16px;
  font-size: 16px;
  color: var(--text-color-2);
}
</style>
