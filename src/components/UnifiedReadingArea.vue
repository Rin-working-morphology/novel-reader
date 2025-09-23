<template>
  <BaseReadingArea
    ref="baseReadingAreaRef"
    :current-file="currentFile"
    :chapters="chapters"
    :current-chapter-index="currentChapterIndex"
    :theme="theme"
    :loading="loading"
    @chapter-changed="handleChapterChanged"
    @scroll="handleScroll"
  >
    <template #content="{ currentChapter, chapters, currentChapterIndex }">
      <!-- 根据文件类型选择渲染器 -->
      <TxtContentRenderer
        v-if="isTxtFile"
        :current-chapter="currentChapter"
      />
      <EpubContentRenderer
        v-else-if="isEpubFile"
        :current-chapter="currentChapter"
        :chapters="chapters"
        :current-chapter-index="currentChapterIndex"
        @chapter-changed="handleChapterChanged"
      />
    </template>
  </BaseReadingArea>
</template>

<script setup lang="ts">
import { watch, nextTick, ref, computed } from "vue";
import BaseReadingArea from "./BaseReadingArea.vue";
import TxtContentRenderer from "./TxtContentRenderer.vue";
import EpubContentRenderer from "./EpubContentRenderer.vue";
import { useReading, useScrollPosition } from "../composables/useReading";
import { type TxtFile } from "../services/fileService";

interface Props {
  currentFile: TxtFile | null;
  currentChapterIndex: number;
  theme: string;
  progress: Record<string, any>;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "chapters-loaded": [chapters: any[]];
  "chapter-changed": [index: number];
  scroll: [position: number];
}>();

const baseReadingAreaRef = ref();
const {
  loading,
  chapters,
  currentChapterIndex,
  jumpToChapter,
  loadTxtFileChapters,
  loadEpubFileChapters,
  loadSingleEpubChapter,
} = useReading();
const { restoreScrollPosition } = useScrollPosition(baseReadingAreaRef);

// 判断文件类型
const isTxtFile = computed(() => {
  return props.currentFile?.path.toLowerCase().endsWith(".txt") || false;
});

const isEpubFile = computed(() => {
  return props.currentFile?.path.toLowerCase().endsWith(".epub") || false;
});

// 监听文件变化，根据类型加载章节
watch(
  () => props.currentFile,
  async (newFile) => {
    if (newFile) {
      await nextTick();

      let loadedChapters;
      if (isTxtFile.value) {
        // 加载TXT文件
        loadedChapters = await loadTxtFileChapters(newFile, props.progress);

        // 设置章节索引但不触发滚动
        currentChapterIndex.value = props.progress.current_chapter || 0;
        emit("chapter-changed", currentChapterIndex.value);

        await nextTick();
        // 恢复滚动位置
        restoreScrollPosition(props.progress.scroll_position);
      } else if (isEpubFile.value) {
        // 加载EPUB文件
        loadedChapters = await loadEpubFileChapters(newFile);
      }

      if (loadedChapters) {
        emit("chapters-loaded", loadedChapters);
      }

      if (!props.progress.scroll_position) {
        setTimeout(() => {
          baseReadingAreaRef.value.checkContentHeight();
        }, 100);
      }
    }
  },
  { immediate: true }
);

// 监听章节变化
watch(
  () => props.currentChapterIndex,
  async (newIndex, oldIndex) => {
    if (isTxtFile.value) {
      // TXT文件的章节切换逻辑
      await nextTick();
      // 只有在用户主动切换章节时才滚动到顶部
      if (newIndex !== props.progress.current_chapter || oldIndex === undefined) {
        // 这是用户主动切换章节，滚动到顶部
        if (baseReadingAreaRef.value) {
          baseReadingAreaRef.value.scrollToPosition(0);
        }
      }
      currentChapterIndex.value = newIndex;
    } else if (isEpubFile.value) {
      // EPUB文件的章节切换逻辑
      if (props.currentFile && newIndex >= 0) {
        // 检查当前章节是否已加载内容
        const currentChapter = chapters.value[newIndex];
        if (!currentChapter?.content) {
          await loadSingleEpubChapter(props.currentFile.path, newIndex);

          await nextTick();
          // 恢复滚动位置
          restoreScrollPosition(props.progress.scroll_position);
        }
      }
      currentChapterIndex.value = newIndex;
    }

    setTimeout(() => {
      baseReadingAreaRef.value.checkContentHeight();
    }, 100);
  },
  { immediate: true }
);

const handleChapterChanged = (index: number) => {
  jumpToChapter(index);
  emit("chapter-changed", index);
};

const handleScroll = (position: number) => {
  emit("scroll", position);
};

// 暴露方法供父组件调用
const scrollToPosition = (position: number) => {
  if (baseReadingAreaRef.value) {
    baseReadingAreaRef.value.scrollToPosition(position);
  }
};

defineExpose({
  contentRef: baseReadingAreaRef,
  scrollToPosition,
});
</script>
