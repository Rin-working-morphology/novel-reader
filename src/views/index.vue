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
          :txt-files="txtFiles"
          :current-file="currentFile"
          @folder-selected="handleFolderSelected"
          @select-file="handleSelectFile"
        />
      </n-message-provider>

      <!-- 主阅读区域 -->
      <n-layout
        has-sider
        sider-placement="right"
      >
        <n-message-provider>
          <!-- TXT文件阅读区域 -->
          <ReadingArea
            v-if="isTxt"
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

          <!-- EPUB文件阅读区域 -->
          <EpubReadingArea
            v-else
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
import ReadingArea from "@/components/ReadingArea.vue";
import ChapterOutline from "@/components/ChapterOutline.vue";
import EpubReadingArea from "@/components/EpubReadingArea.vue";

import { useProgress } from "@/composables/useProgress";
import { FileService, type TxtFile, type Chapter } from "@/services/fileService";

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

const readingProgress = ref<Record<string, number>>({ current_chapter: 0, scroll_position: 0 });

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

// 数据状态
const txtFiles = ref<TxtFile[]>([]);
const chapters = ref<Chapter[]>([]); // 修改这里的类型
const readingAreaRef = ref<InstanceType<typeof ReadingArea>>();

const isTxt = computed(() => {
  if (!currentFile.value) {
    return true;
  }
  return currentFile.value.path.toLowerCase().endsWith(".txt");
});

const debounce = (fn: Function, delay: number) => {
  let timer: number | null = null;
  return (...args: any[]) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      fn(...args);
    }, delay) as unknown as number;
  };
};

// 界面控制方法
const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value;
};

const toggleOutline = () => {
  outlineVisible.value = !outlineVisible.value;
};

// 事件处理方法
const handleFolderSelected = (folderPath: string, files: TxtFile[]) => {
  updateCurrentFolder(folderPath);
  txtFiles.value = files;
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

const handleChaptersLoaded = (loadedChapters: Chapter[]) => {
  // 修改这里的类型
  chapters.value = loadedChapters;
};

const handleChapterChanged = async (index: number) => {
  updateCurrentChapter(index);
  // 不再强制滚动到顶部，除非是用户主动切换章节
  // 如果是从进度恢复，则不应该重置滚动位置
  if (!isRestoringProgress.value) {
    await nextTick();
    if (readingAreaRef.value) {
      readingAreaRef.value.scrollToPosition(0);
    }
  }
};
const debouncedUpdateScrollPosition = debounce(updateScrollPosition, 500);

const handleScroll = (position: number) => {
  debouncedUpdateScrollPosition(position);
};

// 添加一个标志，表示是否正在恢复进度
const isRestoringProgress = ref(false);

// 初始化加载进度
const initializeProgress = async () => {
  const progress = await loadProgress();
  if (progress) {
    isRestoringProgress.value = true; // 设置标志
    updateCurrentFolder(progress.folder_path);

    // 扫描文件夹
    try {
      const files = await FileService.scanTxtFiles(progress.folder_path);
      txtFiles.value = files;

      // 找到上次阅读的文件
      const lastFile = files.find((f) => f.path === progress.current_file);
      if (lastFile) {
        updateCurrentFile(lastFile);
        readingProgress.value = {
          current_chapter: progress.current_chapter,
          scroll_position: progress.scroll_position,
        };

        if (lastFile.path.toLowerCase().endsWith(".epub")) {
          // 只设置章节索引，EPUB组件会自动跳转到对应章节
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

// 生命周期
onMounted(() => {
  initializeProgress();
});
</script>
<style scoped></style>
