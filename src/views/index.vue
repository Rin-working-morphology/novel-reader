<template>
  <n-layout style="height: 100vh">
    <!-- 顶部工具栏 -->
    <HeaderBar
      :sidebar-collapsed="sidebarCollapsed"
      :outline-visible="outlineVisible"
      :theme="props.theme"
      :chapters="chapters"
      :current-chapter-index="currentChapterIndex"
      :title="currentFile?.name || ''"
      @toggle-sidebar="toggleSidebar"
      @toggle-outline="toggleOutline"
      @toggle-theme="toggleTheme"
      @chapter-changed="handleChapterChanged"
    />

    <n-layout
      has-sider
      style="height: calc(100vh - 60px)"
    >
      <!-- 左侧文件列表 -->
      <n-message-provider>
        <FileSidebar
          :collapsed="sidebarCollapsed"
          :current-folder="currentFolder"
          :all-files="allFiles"
          :current-file="currentFile"
          @folder-selected="handleFolderSelected"
          @select-file="handleSelectFile"
        />
      </n-message-provider>

      <n-layout
        has-sider
        sider-placement="right"
      >
        <n-message-provider>
          <!-- 统一阅读区域 -->
          <UnifiedReadingArea
            ref="readingAreaRef"
            :current-file="currentFile"
            :chapters="chapters"
            :theme="props.theme"
            :progress="readingProgress"
            :current-chapter-index="currentChapterIndex"
            @chapters-loaded="handleChaptersLoaded"
            @chapter-changed="handleChapterChanged"
            @scroll="handleScroll"
          />
        </n-message-provider>

        <!-- 右侧大纲 -->
        <ChapterOutline
          :visible="outlineVisible"
          :chapters="chapters"
          :current-chapter-index="currentChapterIndex"
          @jump-to-chapter="handleChapterChanged"
        />
      </n-layout>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, computed } from "vue";
import { NMessageProvider, NLayout } from "naive-ui";

import HeaderBar from "@/components/HeaderBar.vue";
import FileSidebar from "@/components/FileSidebar.vue";
import UnifiedReadingArea from "@/components/UnifiedReadingArea.vue";
import ChapterOutline from "@/components/ChapterOutline.vue";

import { useProgress } from "@/composables/useProgress";
import { FileService, type TxtFile, type Chapter, type ReadingProgress } from "@/services/fileService";
import { debounce } from "lodash-es";

const props = defineProps<{
  theme: string;
}>();

const emit = defineEmits<{
  "toggle-theme": [];
}>();

// 主题管理
const toggleTheme = () => {
  emit("toggle-theme");
};

// 界面状态
const sidebarCollapsed = ref(false);
const outlineVisible = ref(false);

const isRestoringProgress = ref(false);
const readingProgress = ref<ReadingProgress>({ current_chapter: 0, scroll_position: 0 });

// 文件和进度管理
const {
  currentFolder,
  currentFile,
  currentChapterIndex,
  loadProgress,
  updateCurrentFile,
  updateCurrentChapter,
  updateCurrentFolder,
  updateScrollPosition,
} = useProgress();

// 章节数据
const chapters = ref<Chapter[]>([]);
const allFiles = ref<TxtFile[]>([]);

const readingAreaRef = ref();

// 文件夹选择处理
const handleFolderSelected = (folderPath: string, files: TxtFile[]) => {
  updateCurrentFolder(folderPath);
  allFiles.value = files;
};

// 章节加载完成处理
const handleChaptersLoaded = (loadedChapters: Chapter[]) => {
  chapters.value = loadedChapters;
};

// 章节切换处理
const handleChapterChanged = async (index: number) => {
  updateCurrentChapter(index);

  await nextTick();
  // 滚动到顶部
  if (readingAreaRef.value) {
    readingAreaRef.value.scrollToPosition(0);
  }
};

// 滚动处理
const debouncedUpdateScrollPosition = debounce(updateScrollPosition, 500);

const handleScroll = (position: number) => {
  debouncedUpdateScrollPosition(position);
};

// 界面控制方法
const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value;
};

const toggleOutline = () => {
  outlineVisible.value = !outlineVisible.value;
};

const handleSelectFile = (file: TxtFile) => {
  // 如果是同一个文件，不需要重置
  if (currentFile.value && currentFile.value.path === file.path) {
    return;
  }

  // 重置章节状态
  chapters.value = [];
  updateCurrentChapter(0);
  // 重置阅读进度
  readingProgress.value = { current_chapter: 0, scroll_position: 0 };
  updateCurrentFile(file);
};

// 初始化加载进度
const initializeProgress = async () => {
  const progress = await loadProgress();
  if (progress && progress?.folder_path) {
    isRestoringProgress.value = true; // 设置标志
    updateCurrentFolder(progress.folder_path);

    // 扫描文件夹
    try {
      const files = await FileService.scanTxtFiles(progress.folder_path);
      allFiles.value = files;

      // 找到上次阅读的文件
      const lastFile = files.find((f) => f.path === progress.current_file);
      if (lastFile) {
        updateCurrentFile(lastFile);
        readingProgress.value = {
          current_chapter: progress.current_chapter,
          scroll_position: progress.scroll_position,
        };

        if (lastFile.path.toLowerCase().endsWith(".epub")) {
          currentChapterIndex.value = progress.current_chapter;
        }
      }
    } catch (error) {
      console.error("恢复进度失败:", error);
    } finally {
      // 延迟重置标志，确保所有恢复操作完成
      setTimeout(() => {
        isRestoringProgress.value = false;
      }, 1000);
    }
  }
};

onMounted(() => {
  initializeProgress();
});
</script>

<style scoped></style>
