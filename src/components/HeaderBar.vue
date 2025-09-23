<template>
  <n-layout-header
    data-tauri-drag-region
    style="height: 60px; padding: 0 16px"
    bordered
  >
    <div
      class="header-content"
      data-tauri-drag-region
    >
      <div
        class="header-left"
        data-tauri-drag-region
      >
        <n-gradient-text
          data-tauri-drag-region
          :size="16"
          style="font-weight: 600"
          >{{ title }}
        </n-gradient-text>
      </div>

      <!-- 章节导航 -->
      <div
        class="chapter-nav"
        data-tauri-drag-region
        v-if="chapters.length > 1"
      >
        <n-select
          :value="currentChapterIndex"
          :options="chapterOptions"
          @update:value="handleJumpToChapter"
          style="width: 220px"
        />
      </div>

      <div
        class="header-right"
        data-tauri-drag-region
      >
        <n-button
          quaternary
          circle
          v-if="chapters.length > 1"
          @click="handleToggleOutline"
          :render-icon="renderIcon(ListOutline)"
          :type="outlineVisible ? 'primary' : 'default'"
        />
        <n-button
          quaternary
          circle
          @click="$emit('toggle-theme')"
          :render-icon="renderIcon(renderThemeIcon(props.theme))"
        />
        <n-button
          quaternary
          circle
          @click="close"
          :render-icon="renderIcon(Close)"
        />
      </div>
    </div>
  </n-layout-header>
</template>

<script setup lang="ts">
import { NLayoutHeader, NButton, NSelect, NGradientText } from "naive-ui";
import { ListOutline, SunnyOutline, MoonOutline, Remove, Close } from "@vicons/ionicons5";
import { renderIcon } from "../utils/icon";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type Chapter } from "../services/fileService";
import { computed } from "vue";

interface Props {
  sidebarCollapsed: boolean;
  outlineVisible: boolean;
  theme: string;
  title: string;
  chapters: Chapter[];
  currentChapterIndex: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "toggle-outline": [];
  "toggle-theme": [];
  "chapter-changed": [index: number];
}>();

const renderThemeIcon = (theme: string) => {
  if (theme === "default") {
    return SunnyOutline;
  } else {
    return MoonOutline;
  }
};

const chapterOptions = computed(() => {
  return props.chapters.map((chapter, index) => ({
    label: chapter.title,
    value: index,
  }));
});

const handleJumpToChapter = (index: number) => {
  emit("chapter-changed", index);
};

const handleToggleOutline = () => {
  emit("toggle-outline");
};

const close = async () => {
  await getCurrentWindow()?.hide();
};
</script>

<style scoped>
.header-content {
  display: flex;
  align-items: center;
  height: 100%;
  min-width: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  .n-gradient-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
}

.chapter-nav {
  flex-shrink: 1;
  min-width: 220px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}
</style>
