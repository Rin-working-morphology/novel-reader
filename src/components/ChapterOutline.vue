<template>
  <n-layout-sider
    v-if="chapters.length > 1"
    bordered
    :width="250"
    show-trigger="arrow-circle"
    collapse-mode="width"
    :collapsed="!visible"
    :collapsed-width="0"
    :show-collapsed-content="false"
    content-style="padding: 14px 12px; overflow: hidden;"
    @update:collapsed="handleCollapsedChange"
  >
    <div class="outline-content">
      <div class="outline-title">
        <n-text strong>章节目录</n-text>
        <span class="outline-count">{{ currentChapterIndex + 1 }} / {{ chapters.length }}</span>
      </div>
      <n-tree
        ref="treeInstRef"
        class="chapter-tree"
        :data="treeData"
        :selected-keys="selectedKeys"
        :expanded-keys="expandedKeys"
        :virtual-scroll="true"
        :node-props="getNodeProps"
        :checkable="false"
        block-line
        @update:expanded-keys="handleExpandedKeysChange"
      />
    </div>
  </n-layout-sider>
</template>

<script setup lang="ts">
import { NLayoutSider, NText, NTree, type TreeInst, type TreeOption } from "naive-ui";
import { BookOutline, Book } from "@vicons/ionicons5";
import { ref, computed, watch, nextTick } from "vue";

import { renderIcon } from "@/utils/icon";
import { type Chapter } from "@/services/fileService";

type TreeKey = string | number;
type ChapterTreeNode = TreeOption & {
  key: number;
  chapterIndex: number;
  chapterLevel: number;
  children?: ChapterTreeNode[];
};

interface Props {
  visible: boolean;
  chapters: Chapter[];
  currentChapterIndex: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "jump-to-chapter": [index: number];
  "update-visible": [visible: boolean];
}>();

const treeInstRef = ref<TreeInst | null>(null);
const expandedKeys = ref<TreeKey[]>([]);

const handleCollapsedChange = (collapsed: boolean) => {
  emit("update-visible", !collapsed);
};

// 滚动到指定章节
const scrollToChapter = (index: number) => {
  const visibleIndex = visibleNodeIndexMap.value.get(index) ?? index;
  if (treeInstRef.value && visibleIndex >= 20 && index < props.chapters.length) {
    treeInstRef.value.scrollTo({
      index: Math.min(visibleIndex + 10, visibleNodeCount.value - 1),
    });
  }
};

// 监听当前章节变化，自动滚动到对应位置
watch(
  () => props.currentChapterIndex,
  (newIndex) => {
    if (newIndex >= 0) {
      // 使用nextTick确保DOM更新完成后再滚动
      nextTick(() => {
        expandCurrentChapterParents(newIndex);
        scrollToChapter(newIndex);
      });
    }
  }
);

// 暴露方法给父组件
defineExpose({
  scrollToChapter,
});

const getChapterLevel = (chapter: Chapter) => {
  const level = Number(chapter.level);
  return Number.isFinite(level) && level > 0 ? level : 1;
};

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

// 将后端输出的 parent_index 扁平目录转换为树形目录
const treeData = computed<ChapterTreeNode[]>(() => {
  const nodes = props.chapters.map((chapter, index): ChapterTreeNode => ({
    key: index,
    label: chapter.title || `第${index + 1}章`,
    chapterIndex: index,
    chapterLevel: getChapterLevel(chapter),
    children: [],
    prefix: () => renderTreeIcon(index),
  }));
  const roots: ChapterTreeNode[] = [];

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
      node.isLeaf = true;
      delete node.children;
    }
  });

  return roots;
});

const selectedKeys = computed(() => [props.currentChapterIndex]);
const expandableKeys = computed<TreeKey[]>(() => {
  const keys: TreeKey[] = [];
  const collect = (nodes: ChapterTreeNode[]) => {
    nodes.forEach((node) => {
      if (node.children?.length) {
        keys.push(node.key);
        collect(node.children);
      }
    });
  };

  collect(treeData.value);
  return keys;
});

const visibleNodeIndexMap = computed(() => {
  const map = new Map<number, number>();
  const expandedKeySet = new Set(expandedKeys.value);
  let visibleIndex = 0;

  const collect = (nodes: ChapterTreeNode[]) => {
    nodes.forEach((node) => {
      map.set(node.chapterIndex, visibleIndex);
      visibleIndex += 1;

      if (node.children?.length && expandedKeySet.has(node.key)) {
        collect(node.children);
      }
    });
  };

  collect(treeData.value);
  return map;
});

const visibleNodeCount = computed(() => visibleNodeIndexMap.value.size);

const getAncestorKeys = (index: number) => {
  const keys: TreeKey[] = [];
  const visited = new Set<number>();
  let parentIndex = props.chapters[index] ? getParentIndex(props.chapters[index], index) : null;

  while (parentIndex !== null && !visited.has(parentIndex)) {
    keys.push(parentIndex);
    visited.add(parentIndex);
    parentIndex = props.chapters[parentIndex]
      ? getParentIndex(props.chapters[parentIndex], parentIndex)
      : null;
  }

  return keys;
};

const expandCurrentChapterParents = (index: number) => {
  const mergedKeys = new Set(expandedKeys.value);
  getAncestorKeys(index).forEach((key) => mergedKeys.add(key));
  expandedKeys.value = Array.from(mergedKeys);
};

const handleExpandedKeysChange = (keys: TreeKey[]) => {
  expandedKeys.value = keys;
};

watch(
  () => props.chapters,
  () => {
    expandedKeys.value = expandableKeys.value;
    expandCurrentChapterParents(props.currentChapterIndex);
  },
  { immediate: true }
);

// 获取节点属性
const getNodeProps = (info: any) => {
  const index = info.option.key;
  return {
    onClick: () => {
      emit("jump-to-chapter", index);
    },
    class: props.currentChapterIndex === index ? "active-chapter" : "",
  };
};

// 自定义渲染标签
const renderTreeIcon = (index: number) =>
  props.currentChapterIndex === index ? renderIcon(Book)() : renderIcon(BookOutline)();
</script>

<style scoped>
.outline-content {
  height: 100%;
  width: 226px;
  min-width: 226px;
  overflow: hidden;
}

.outline-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  padding: 0 4px;
  line-height: 1.4;
}

.outline-count {
  flex-shrink: 0;
  color: var(--chapter-nav-muted);
  font-size: 12px;
  font-weight: 500;
}

.chapter-tree {
  height: calc(100vh - 132px);
  padding: 2px;
}

.chapter-tree :deep(.n-tree-node-wrapper .n-tree-node-switcher.n-tree-node-switcher--hide) {
  display: none;
}

.chapter-tree :deep(.n-tree-node) {
  min-height: var(--chapter-nav-height);
  margin-bottom: 4px;
  color: var(--chapter-nav-text);
}

.chapter-tree :deep(.n-tree-node-content) {
  min-height: var(--chapter-nav-height);
  padding: var(--chapter-nav-y) var(--chapter-nav-x);
  border: 1px solid transparent;
  border-radius: var(--chapter-nav-radius);
  color: inherit;
  transition: var(--chapter-nav-transition);
}

.chapter-tree :deep(.n-tree-node-content:hover) {
  border-color: var(--chapter-nav-border);
  background-color: var(--chapter-nav-surface-hover);
}

.chapter-tree :deep(.n-tree-node-content__text) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  font-weight: 500;
  letter-spacing: 0;
}

.chapter-tree :deep(.n-tree-node-content__prefix) {
  color: var(--chapter-nav-muted);
}

.chapter-tree :deep(.active-chapter .n-tree-node-content),
.chapter-tree :deep(.n-tree-node-content.active-chapter),
.chapter-tree :deep(.active-chapter),
.chapter-tree :deep(.n-tree-node--selected .n-tree-node-content) {
  border-color: color-mix(in srgb, var(--chapter-nav-accent) 36%, var(--chapter-nav-border));
  background-color: var(--chapter-nav-surface-active);
  color: var(--chapter-nav-accent);
}

.chapter-tree :deep(.active-chapter .n-tree-node-content__prefix),
.chapter-tree :deep(.n-tree-node-content.active-chapter .n-tree-node-content__prefix),
.chapter-tree :deep(.n-tree-node--selected .n-tree-node-content__prefix) {
  color: var(--chapter-nav-accent);
}

.chapter-tree :deep(.n-tree-node-content:focus-visible) {
  outline: none;
  box-shadow: var(--chapter-nav-focus);
}

@media (prefers-reduced-motion: reduce) {
  .chapter-tree :deep(.n-tree-node-content) {
    transition: none;
  }
}
</style>
