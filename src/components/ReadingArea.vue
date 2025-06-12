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
  import { computed, ref, watch, nextTick } from 'vue';
  import { NLayoutContent, NEmpty, NIcon, NScrollbar, useMessage, NButton, NSpin } from 'naive-ui';
  import { BookOutline, ArrowForwardOutline, ArrowBackOutline } from '@vicons/ionicons5';
  import { FileService, type TxtFile, type Chapter } from '../services/fileService';

  interface Props {
    currentFile: TxtFile | null;
    chapters: Chapter[];
    currentChapterIndex: number;
    theme: string;
    progress: Record<string, number>;
  }

  const props = defineProps<Props>();
  const emit = defineEmits<{
    'chapters-loaded': [chapters: Chapter[]];
    'chapter-changed': [index: number];
    scroll: [position: number];
  }>();

  const message = useMessage();
  const contentRef = ref();
  const loading = ref(false);

  const currentChapter = computed(() => {
    return props.chapters[props.currentChapterIndex] || null;
  });

  // 监听文件变化，自动加载章节
  watch(
    () => props.currentFile,
    async (newFile) => {
      if (newFile) {
        await nextTick();
        await loadFileChapters(newFile);
      }
    },
    { immediate: true }
  );

  // 监听章节变化，滚动到顶部
  watch(
    () => props.currentChapterIndex,
    async (newIndex, oldIndex) => {
      await nextTick();
      // 只有在用户主动切换章节时才滚动到顶部
      // 如果是初始加载或恢复进度，则不应该重置滚动位置
      if (newIndex !== props.progress.current_chapter || oldIndex === undefined) {
        // 这是用户主动切换章节，滚动到顶部
        if (contentRef.value) {
          contentRef.value.scrollTo({ top: 0 });
        }
      }
    }
  );

  const loadFileChapters = async (file: TxtFile) => {
    try {
      loading.value = true;

      // 读取文件内容
      const content = await FileService.readTxtFile(file.path);

      // 解析章节
      const parsedChapters = await FileService.parseChapters(content);

      emit('chapters-loaded', parsedChapters);

      message.success(`已加载 ${parsedChapters.length} 个章节`);

      // 设置章节索引但不触发滚动
      emit('chapter-changed', props.progress.current_chapter);

      await nextTick();
      // 恢复滚动位置
      if (contentRef.value && props.progress.scroll_position > 0) {
        // 使用延迟确保滚动位置恢复在其他操作之后
        setTimeout(() => {
          if (contentRef.value) {
            contentRef.value.scrollTo({ top: props.progress.scroll_position });
          }
        }, 100);
      }
    } catch (error) {
      message.error('读取文件失败: ' + error);
    } finally {
      loading.value = false;
    }
  };

  const handleJumpToChapter = (index: number) => {
    emit('chapter-changed', index);
  };

  const handleScroll = () => {
    if (contentRef.value?.scrollbarInstRef) {
      emit('scroll', contentRef.value?.scrollbarInstRef?.containerScrollTop);
    }
  };

  // 暴露方法供父组件调用
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

<style>
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

  .chapter-nav {
    padding: 16px;
    border-bottom: 1px solid var(--n-border-color);
  }

  .content-text {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    line-height: 1.8;
  }

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

  .frosted-glass {
    background-color: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border-radius: 8px;
    /* box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1); */
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
