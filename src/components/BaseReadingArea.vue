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
          <!-- 顶部遮罩层导航 -->
          <div
            class="chapter-overlay top-overlay"
            v-if="showTopNavigation"
          >
            <div
              class="overlay-content"
              @click="handleJumpToChapter(currentChapterIndex - 1)"
            >
              <n-icon size="24">
                <ArrowBackOutline />
              </n-icon>
              <span class="overlay-text">返回上一章</span>
              <span class="overlay-chapter-title">{{ chapters[currentChapterIndex - 1]?.title }}</span>
            </div>
          </div>

          <!-- 内容渲染插槽 -->
          <slot
            name="content"
            :current-chapter="currentChapter"
            :chapters="chapters"
            :current-chapter-index="currentChapterIndex"
          />

          <!-- 底部遮罩层导航 -->
          <div
            class="chapter-overlay bottom-overlay"
            v-if="showBottomNavigation"
          >
            <div
              class="overlay-content"
              @click="handleJumpToChapter(currentChapterIndex + 1)"
            >
              <span class="overlay-chapter-title">{{ chapters[currentChapterIndex + 1]?.title }}</span>
              <span class="overlay-text">前往下一章</span>
              <n-icon size="24">
                <ArrowForwardOutline />
              </n-icon>
            </div>
          </div>
        </n-scrollbar>
      </div>
    </div>
  </n-layout-content>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
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
const showTopOverlay = ref(false);
const showBottomOverlay = ref(false);

const currentChapter = computed(() => {
  return props.chapters[props.currentChapterIndex] || null;
});

const showTopNavigation = computed(() => {
  return props.chapters.length > 1 && props.currentChapterIndex > 0 && showTopOverlay.value;
});

const showBottomNavigation = computed(() => {
  return props.chapters.length > 1 && props.currentChapterIndex < props.chapters.length - 1 && showBottomOverlay.value;
});

// 检查内容是否足够长以触发滚动
const checkContentHeight = () => {
  if (contentRef.value?.scrollbarInstRef) {
    if (props.chapters.length > 1 && props.currentChapterIndex > 0) {
      showTopOverlay.value = true;
    }
    const scrollHeight = contentRef.value.scrollbarInstRef.containerRef.scrollHeight;
    const clientHeight = contentRef.value.scrollbarInstRef.containerRef.clientHeight;

    // 如果内容高度小于等于容器高度，说明没有滚动条，默认显示覆盖层
    if (
      scrollHeight <= clientHeight &&
      props.chapters.length > 1 &&
      props.currentChapterIndex < props.chapters.length - 1
    ) {
      // 只有在有多章节且有下一章时才显示底部遮罩层
      showBottomOverlay.value = true;
    } else {
      showBottomOverlay.value = false;
    }
  }
};

const handleJumpToChapter = (index: number) => {
  emit("chapter-changed", index);
};

const handleScroll = () => {
  if (contentRef.value?.scrollbarInstRef) {
    const scrollTop = contentRef.value.scrollbarInstRef.containerScrollTop;
    const scrollHeight = contentRef.value.scrollbarInstRef.containerRef.scrollHeight;
    const clientHeight = contentRef.value.scrollbarInstRef.containerRef.clientHeight;

    showTopOverlay.value = scrollTop < 50;
    showBottomOverlay.value = scrollHeight - scrollTop - clientHeight < 50;

    emit("scroll", scrollTop);
  }
};

// 暴露方法供父组件调用
const scrollToPosition = (position: number) => {
  if (contentRef.value) {
    contentRef.value.scrollTo({ top: position });
  }
};

onMounted(() => {});

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
  padding: 120px 24px;
  line-height: 1.8;
}

.frosted-glass {
  background-color: rgba(var(--n-color), 0.1);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  border-radius: 8px;
}

.chapter-overlay {
  position: absolute;
  left: 0;
  right: 0;
  height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
  z-index: 10;
}

.top-overlay {
  top: 0;
  background: linear-gradient(180deg, color-mix(in srgb, var(--n-text-color) 10%, transparent) 0%, transparent 100%);
}

.bottom-overlay {
  bottom: 0;
  background: linear-gradient(0deg, color-mix(in srgb, var(--n-text-color) 10%, transparent) 0%, transparent 100%);
}

.overlay-content {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--n-text-color);
  padding: 12px 24px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
  cursor: pointer;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.overlay-content:hover {
  background: linear-gradient(0deg, color-mix(in srgb, var(--n-text-color) 20%, transparent) 0%, transparent 100%);
}

.overlay-text {
  font-size: 14px;
  font-weight: 500;
}

.overlay-chapter-title {
  font-size: 12px;
  opacity: 0.8;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
