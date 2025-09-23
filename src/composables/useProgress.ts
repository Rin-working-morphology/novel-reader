import { ref, onMounted, onUnmounted } from "vue";
import { FileService, type ReadingProgress, type TxtFile } from "../services/fileService";

export function useProgress() {
  const currentFolder = ref("");
  const currentFile = ref<TxtFile | null>(null);
  const currentChapterIndex = ref(0);
  const scrollPosition = ref(0);

  const saveProgress = async () => {
    if (!currentFile.value) return;

    const progress: ReadingProgress = {
      folder_path: currentFolder.value,
      current_file: currentFile.value?.path,
      current_chapter: currentChapterIndex.value,
      scroll_position: scrollPosition.value,
      last_read_time: new Date().toISOString(),
    };
    try {
      await FileService.saveProgress(progress);
    } catch (error) {
      console.error("保存进度失败:", error);
    }
  };

  const loadProgress = async () => {
    try {
      const progress = await FileService.loadProgress();
      if (progress) {
        return progress;
      }
    } catch (error) {
      console.error("加载进度失败:", error);
    }
    return null;
  };

  const updateScrollPosition = (position: number) => {
    scrollPosition.value = position;
    saveProgress();
  };

  const updateCurrentFile = (file: TxtFile | null) => {
    currentFile.value = file;
  };

  const updateCurrentChapter = (index: number) => {
    currentChapterIndex.value = index;
  };

  const updateCurrentFolder = (folder: string) => {
    currentFolder.value = folder;
  };

  // 窗口关闭前保存进度
  const handleBeforeUnload = () => {
    saveProgress();
  };

  onMounted(() => {
    window.addEventListener("beforeunload", handleBeforeUnload);
  });

  onUnmounted(() => {
    saveProgress();
    window.removeEventListener("beforeunload", handleBeforeUnload);
  });

  return {
    currentFolder,
    currentFile,
    currentChapterIndex,
    scrollPosition,
    saveProgress,
    loadProgress,
    updateScrollPosition,
    updateCurrentFile,
    updateCurrentChapter,
    updateCurrentFolder,
  };
}
