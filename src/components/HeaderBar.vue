<template>
  <n-layout-header
    data-tauri-drag-region
    style="height: 60px; padding: 0 16px"
    bordered
  >
    <div class="header-content" data-tauri-drag-region>
      <div class="header-left" data-tauri-drag-region>
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
        <n-tree-select
          class="chapter-tree-select"
          :value="currentChapterIndex"
          :options="chapterOptions"
          :indent="18"
          :to="'body'"
          default-expand-all
          filterable
          placeholder="选择章节"
          aria-label="章节导航"
          @update:value="handleJumpToChapter"
        />
      </div>

      <div class="header-right" data-tauri-drag-region>
        <n-dropdown :options="dropDownOptions" @select="handleMenuSelect">
          <n-button quaternary circle class="menu-button">
            <n-icon><SettingsOutline /></n-icon>
          </n-button>
        </n-dropdown>

        <n-button quaternary circle @click="close">
          <n-icon><Close /></n-icon>
        </n-button>
      </div>
    </div>
  </n-layout-header>
</template>

<script setup lang="ts">
import { computed } from "vue";
import {
  NLayoutHeader,
  NButton,
  NTreeSelect,
  NDropdown,
  NIcon,
  NGradientText,
  type TreeSelectOption,
} from "naive-ui";
import {
  SunnyOutline,
  MoonOutline,
  Close,
  SettingsOutline,
} from "@vicons/ionicons5";
import { renderIcon } from "../utils/icon";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { type Chapter } from "../services/fileService";

interface Props {
  theme: string;
  title: string;
  chapters: Chapter[];
  currentChapterIndex: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
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

const dropDownOptions = computed(() => [
  {
    label: (props.theme === "default" ? "浅色" : "深色") + "模式",
    key: "theme",
    icon: renderIcon(renderThemeIcon(props.theme)),
  },
]);

const getParentIndex = (chapter: Chapter, index: number) => {
  const parentIndex = chapter.parent_index;

  if (
    typeof parentIndex === "number" &&
    Number.isInteger(parentIndex) &&
    parentIndex >= 0 &&
    parentIndex < index &&
    parentIndex < props.chapters.length &&
    parentIndex !== index
  ) {
    return parentIndex;
  }

  return null;
};

const chapterOptions = computed<TreeSelectOption[]>(() => {
  const nodes = props.chapters.map(
    (chapter, index): TreeSelectOption => ({
      key: index,
      label: chapter.title || `第${index + 1}章`,
      children: [],
    }),
  );
  const roots: TreeSelectOption[] = [];

  nodes.forEach((node, index) => {
    const parentIndex = getParentIndex(props.chapters[index], index);

    if (parentIndex !== null && nodes[parentIndex]) {
      nodes[parentIndex].children?.push(node);
    } else {
      roots.push(node);
    }
  });

  nodes.forEach((node) => {
    if (!node.children?.length) {
      delete node.children;
    }
  });

  return roots;
});

const handleJumpToChapter = (
  value: string | number | Array<string | number> | null,
) => {
  if (typeof value === "number") {
    emit("chapter-changed", value);
  }
};

const close = async () => {
  await getCurrentWindow()?.hide();
};

const handleMenuSelect = (key: string) => {
  if (key === "theme") {
    emit("toggle-theme");
  }
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
}

.book-title {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--n-primary-color);
  font-size: 16px;
  font-weight: 600;
  line-height: 1.4;
}

.chapter-nav {
  flex: 0 1 280px;
  min-width: 180px;
  max-width: 320px;
}

.chapter-tree-select {
  width: 100%;
}

.chapter-tree-select :deep(.n-base-selection) {
  --n-height: var(--chapter-nav-height);
  --n-border-radius: var(--chapter-nav-radius);
  --n-border: 1px solid var(--chapter-nav-border);
  --n-border-hover: 1px solid var(--chapter-nav-accent);
  --n-border-active: 1px solid var(--chapter-nav-accent);
  --n-border-focus: 1px solid var(--chapter-nav-accent);
  --n-color: var(--chapter-nav-surface);
  --n-color-active: var(--chapter-nav-surface);
  --n-box-shadow-focus: var(--chapter-nav-focus);
  --n-text-color: var(--chapter-nav-text);
  font-size: 13px;
  transition: var(--chapter-nav-transition);
}

.chapter-tree-select :deep(.n-base-selection:hover) {
  --n-color: var(--chapter-nav-surface-hover);
}

.chapter-tree-select :deep(.n-base-selection-label) {
  padding-inline: var(--chapter-nav-x);
}

.chapter-tree-select :deep(.n-base-selection-input),
.chapter-tree-select :deep(.n-base-selection-placeholder) {
  font-weight: 500;
  letter-spacing: 0;
}

.chapter-tree-select :deep(.n-base-selection-placeholder) {
  color: var(--chapter-nav-muted);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  margin-left: 8px;
}

.menu-button {
  transition: transform 0.18s ease;
}
.menu-button:hover {
  transform: rotate(-90deg);
}

@media (prefers-reduced-motion: reduce) {
  .menu-button,
  .chapter-tree-select :deep(.n-base-selection) {
    transition: none;
  }

  .menu-button:hover {
    transform: none;
  }
}
</style>
