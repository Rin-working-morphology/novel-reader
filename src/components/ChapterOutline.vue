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
        :virtual-scroll="true"
        :node-props="getNodeProps"
        :checkable="false"
        block-line
      />
    </div>
  </n-layout-sider>
</template>

<script setup lang="ts">
import { NLayoutSider, NText, NTree } from "naive-ui";
import { BookOutline, Book } from "@vicons/ionicons5";
import { ref, computed, watch, nextTick } from "vue";

import { renderIcon } from "@/utils/icon";

interface Chapter {
  title: string;
  content: string;
  start_pos: number;
  end_pos: number;
}

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

const treeInstRef = ref<InstanceType<typeof NTree> | null>(null);

const handleCollapsedChange = (collapsed: boolean) => {
  emit("update-visible", !collapsed);
};

// 滚动到指定章节
const scrollToChapter = (index: number) => {
  if (treeInstRef.value && index >= 20 && index < props.chapters.length) {
    if (index + 10 < props.chapters.length) {
      treeInstRef.value.scrollTo({ index: index + 10 });
    } else {
      treeInstRef.value.scrollTo({ index: props.chapters.length - 1 });
    }
  }
};

// 监听当前章节变化，自动滚动到对应位置
watch(
  () => props.currentChapterIndex,
  (newIndex) => {
    if (newIndex >= 0) {
      // 使用nextTick确保DOM更新完成后再滚动
      nextTick(() => {
        scrollToChapter(newIndex);
      });
    }
  }
);

// 暴露方法给父组件
defineExpose({
  scrollToChapter,
});

// 将章节数据转换为树形数据
const treeData = computed(() => {
  return props.chapters.map((chapter, index) => ({
    key: index,
    label: chapter.title,
    isLeaf: true,
    children: [],
    prefix: () => renderTreeIcon(index),
  }));
});

const selectedKeys = computed(() => [props.currentChapterIndex]);

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
