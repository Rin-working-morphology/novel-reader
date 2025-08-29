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

        <n-scrollbar
          ref="contentRef"
          :content-class="`content-text ${props.theme === 'default' ? 'frosted-glass' : ''}`"
          @scroll="handleScroll"
        >
          <!-- 顶部章节导航 -->
          <div
            class="chapter-navigation top"
            v-if="chapters.length > 1 && props.currentChapterIndex > 0"
          >
            <n-button
              @click="handleJumpToChapter(props.currentChapterIndex - 1)"
              size="small"
            >
              <template #icon>
                <n-icon><ArrowBackOutline /></n-icon>
              </template>
              返回上一章：{{ chapters[props.currentChapterIndex - 1]?.title }}
            </n-button>
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

          <!-- 底部章节导航 -->
          <div
            class="chapter-navigation bottom"
            v-if="chapters.length > 1 && props.currentChapterIndex < chapters.length - 1"
          >
            <n-button
              @click="handleJumpToChapter(props.currentChapterIndex + 1)"
              size="small"
            >
              前往下一章：{{ chapters[props.currentChapterIndex + 1]?.title }}
              <template #icon>
                <n-icon><ArrowForwardOutline /></n-icon>
              </template>
            </n-button>
          </div>
        </n-scrollbar>
      </div>
    </div>
  </n-layout-content>
</template>

<script setup lang="ts">
import { computed, ref, watch, nextTick } from "vue";
import { NLayoutContent, NEmpty, NIcon, NScrollbar, NButton, NRadioGroup, NRadio, useMessage, NSpin } from "naive-ui";
import { BookOutline, ArrowForwardOutline, ArrowBackOutline } from "@vicons/ionicons5";
import DOMPurify from "dompurify";
import { FileService, type TxtFile, type Chapter, RenderMode } from "../services/fileService";

interface Props {
  currentFile: TxtFile | null;
  chapters: Chapter[];
  currentChapterIndex: number;
  theme: string;
  progress: Record<string, any>;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "chapters-loaded": [chapters: Chapter[]];
  "chapter-changed": [index: number];
  scroll: [position: number];
}>();

const message = useMessage();
const contentRef = ref<any>();
const loading = ref(false);
const renderMode = ref<RenderMode>(RenderMode.HTML);

const currentChapter = computed(() => {
  return props.chapters[props.currentChapterIndex] || null;
});

// 安全的HTML内容
const sanitizedHtmlContent = computed(() => {
  if (!currentChapter.value?.html_content) return "";

  // 使用DOMPurify清理HTML，但保留图片
  return DOMPurify.sanitize(currentChapter.value.html_content, {
    ALLOWED_TAGS: ["p", "div", "span", "h1", "h2", "h3", "h4", "h5", "h6", "br", "img", "strong", "em", "u", "i", "b"],
    ALLOWED_ATTR: ["src", "alt", "title", "class", "style"],
    ALLOW_DATA_ATTR: false,
  });
});

watch(
  () => props.currentFile,
  async (newFile) => {
    if (newFile && newFile.path.toLowerCase().endsWith(".epub")) {
      await nextTick();
      await loadChapters(newFile);
    }
  },
  { immediate: true }
);

watch(
  () => props.currentChapterIndex,
  async (newIndex, oldIndex) => {
    await nextTick();
    const progressChapter = props.progress?.current_chapter;
    if (newIndex !== progressChapter || oldIndex === undefined) {
      if (contentRef.value) {
        contentRef.value.scrollTo({ top: 0 });
      }
    }
  },
  { immediate: true }
);

const loadChapters = async (file: TxtFile) => {
  try {
    loading.value = true;

    // 首先获取章节信息列表
    const chaptersInfo = await FileService.getEpubInfo(file.path);

    // 创建轻量级章节列表（只包含标题和索引）
    const lightweightChapters = chaptersInfo.map((info) => ({
      title: info.title,
      content: "", // 暂时为空
      html_content: "",
      start_pos: 0,
      end_pos: 0,
      images: {},
      index: info.index, // 添加索引用于按需加载
    }));

    emit("chapters-loaded", lightweightChapters);
    message.success(`已加载 ${chaptersInfo.length} 个章节目录`);

    // 加载第一章内容
    await loadSingleChapter(file.path, 0);
  } catch (error) {
    message.error("读取EPUB文件失败: " + error);
  } finally {
    loading.value = false;
  }
};

// 按需加载单个章节的函数
const loadSingleChapter = async (filePath: string, chapterIndex: number) => {
  try {
    loading.value = true;
    const chapter = await FileService.loadEpubChapter(filePath, chapterIndex);

    // 更新当前章节的内容
    if (props.chapters[chapterIndex]) {
      props.chapters[chapterIndex].content = chapter.content;
      props.chapters[chapterIndex].html_content = chapter.html_content;
    }

    return chapter;
  } catch (error) {
    message.error(`加载第${chapterIndex + 1}章失败: ` + error);
    throw error;
  } finally {
    loading.value = false;
  }
};

// 监听章节变化，按需加载内容
watch(
  () => props.currentChapterIndex,
  async (newIndex) => {
    if (props.currentFile && newIndex >= 0) {
      // 检查当前章节是否已加载内容
      const currentChapter = props.chapters[newIndex];
      if (!currentChapter?.content) {
        await loadSingleChapter(props.currentFile.path, newIndex);
      }
    }
  },
  { immediate: true }
);

const handleJumpToChapter = (index: number) => {
  emit("chapter-changed", index);
};

const handleScroll = () => {
  if (contentRef.value?.scrollbarInstRef) {
    emit("scroll", contentRef.value?.scrollbarInstRef?.containerScrollTop);
  }
};

const scrollToPosition = (position: number) => {
  if (contentRef.value) {
    contentRef.value.scrollTo({ top: position });
  }
};

defineExpose({
  contentRef,
  scrollToPosition,
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

/* 图片样式 */
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

/* 其他现有样式保持不变 */
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
  color: var(--n-text-color);
}

.content-area {
  flex-grow: 1;
  overflow: hidden;
}

.content-text {
  padding: 20px;
  font-family: "LXGW WenKai Screen R", "Microsoft YaHei", sans-serif;
  line-height: 1.8;
  color: var(--n-text-color);
}

.chapter-title {
  font-size: 24px;
  font-weight: bold;
  margin-bottom: 20px;
  text-align: center;
  color: var(--n-text-color);
}

.chapter-content {
  font-size: 16px;
  white-space: pre-wrap;
  word-wrap: break-word;
  color: var(--n-text-color);
}

.frosted-glass {
  background-color: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 8px;
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
