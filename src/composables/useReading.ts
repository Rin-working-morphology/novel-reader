import { ref, watch, nextTick, computed, Ref } from "vue";
import { useMessage } from "naive-ui";
import { FileService, type TxtFile, type Chapter } from "../services/fileService";

export function useReading() {
  const message = useMessage();
  const loading = ref(false);
  const chapters = ref<Chapter[]>([]);
  const currentChapterIndex = ref(0);

  const currentChapter = computed(() => {
    return chapters.value[currentChapterIndex.value] || null;
  });

  // 加载TXT文件章节
  const loadTxtFileChapters = async (file: TxtFile, progress: Record<string, number>) => {
    try {
      loading.value = true;

      // 读取文件内容
      const content = await FileService.readTxtFile(file.path);

      // 解析章节
      const parsedChapters = await FileService.parseChapters(content);
      chapters.value = parsedChapters;

      message.success(`已加载 ${parsedChapters.length} 个章节`);

      // 设置章节索引
      currentChapterIndex.value = progress.current_chapter || 0;

      return parsedChapters;
    } catch (error) {
      message.error("读取文件失败: " + error);
      throw error;
    } finally {
      loading.value = false;
    }
  };

  // 加载EPUB文件章节
  const loadEpubFileChapters = async (file: TxtFile) => {
    try {
      loading.value = true;

      // 获取增强的章节信息列表
      const chaptersInfo = await FileService.getEpubInfo(file.path);

      // 创建包含层级信息的轻量级章节列表
      const lightweightChapters = chaptersInfo.map((info) => ({
        title: info.title,
        content: "", // 暂时为空
        html_content: "",
        start_pos: 0,
        end_pos: 0,
        images: {},
        index: info.index,
        level: info.level, // 章节层级
        parent_index: info.parent_index, // 父章节索引
        toc_entry: undefined,
        detection_method: info.detection_method, // 检测方法
      }));

      chapters.value = lightweightChapters;
      message.success(`已加载 ${chaptersInfo.length} 个章节目录（包含层级信息）`);

      // 加载第一章内容
      await loadSingleEpubChapter(file.path, 0);

      return lightweightChapters;
    } catch (error) {
      message.error("读取EPUB文件失败: " + error);
      throw error;
    } finally {
      loading.value = false;
    }
  };

  // 按需加载单个EPUB章节
  const loadSingleEpubChapter = async (filePath: string, chapterIndex: number) => {
    try {
      loading.value = true;
      const chapter = await FileService.loadEpubChapter(filePath, chapterIndex);

      // 更新当前章节的内容，包含增强信息
      if (chapters.value[chapterIndex]) {
        chapters.value[chapterIndex].content = chapter.content;
        chapters.value[chapterIndex].html_content = chapter.html_content;
        chapters.value[chapterIndex].level = chapter.level || 1;
        chapters.value[chapterIndex].parent_index = chapter.parent_index;
        chapters.value[chapterIndex].toc_entry = chapter.toc_entry;
        chapters.value[chapterIndex].detection_method = chapter.detection_method;
      }

      return chapter;
    } catch (error) {
      message.error(`加载第${chapterIndex + 1}章失败: ` + error);
      throw error;
    } finally {
      loading.value = false;
    }
  };

  // 跳转到指定章节
  const jumpToChapter = (index: number) => {
    if (index >= 0 && index < chapters.value.length) {
      currentChapterIndex.value = index;
    }
  };

  return {
    loading,
    chapters,
    currentChapterIndex,
    currentChapter,
    loadTxtFileChapters,
    loadEpubFileChapters,
    loadSingleEpubChapter,
    jumpToChapter,
  };
}

// 滚动位置管理
export function useScrollPosition(baseReadingAreaRef: Ref<any | null>) {
  const contentRef = computed(() => baseReadingAreaRef.value?.contentRef);
  const scrollToPosition = (position: number) => {
    if (contentRef.value) {
      contentRef.value.scrollTo({ top: position });
    }
  };

  const restoreScrollPosition = (position: number, delay = 100) => {
    if (contentRef.value && position > 0) {
      setTimeout(() => {
        if (contentRef.value) {
          contentRef.value.scrollTo({ top: position });
        }
      }, delay);
    }
  };

  return {
    contentRef,
    scrollToPosition,
    restoreScrollPosition,
  };
}
