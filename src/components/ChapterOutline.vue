<template>
  <n-layout-sider
    v-if="visible && chapters.length > 1"
    bordered
    :width="250"
    show-trigger="arrow-circle"
    collapse-mode="width"
    :collapsed="collapsed"
    content-style="padding: 16px;"
    :collapsed-width="64"
    @update:collapsed="toggleCollapse"
  >
    <div class="outline-content">
      <n-text
        style="font-weight: 600; margin-bottom: 16px; display: block"
        v-if="!collapsed"
      >
        章节目录
      </n-text>
      <n-tree
        ref="treeInstRef"
        :data="treeData"
        :virtual-scroll="true"
        :node-props="getNodeProps"
        :checkable="false"
        block-line
        style="padding: 4px; height: calc(100vh - 140px)"
      />
    </div>
  </n-layout-sider>
</template>

<script setup lang="ts">
import { NLayoutSider, NText, NTree, NTooltip } from "naive-ui";
import { BookOutline, Book } from "@vicons/ionicons5";
import { ref, computed, h, watch, nextTick } from "vue";

import { renderIcon } from "@/utils/icon";

interface Chapter {
  title: string;
  content: string;
  start_pos: number;
  end_pos: number;
}

interface Props {
  visible: boolean;
  collapsed: boolean;
  chapters: Chapter[];
  currentChapterIndex: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  "jump-to-chapter": [index: number];
  "toggle-collapse": [collapsed: boolean];
}>();

const treeInstRef = ref<InstanceType<typeof NTree> | null>(null);

const toggleCollapse = (collapse: boolean) => {
  emit("toggle-collapse", collapse);
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
    label: props.collapsed ? "" : chapter.title,
    isLeaf: true,
    children: [],
    prefix: () => renderTreeIcon(chapter.title, index),
  }));
});

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
const renderTreeIcon = (label: string, index: number) =>
  h(
    NTooltip,
    { trigger: "hover", placement: "left", disabled: !props.collapsed },
    {
      trigger: props.currentChapterIndex === index ? renderIcon(Book) : renderIcon(BookOutline),
      default: () => label,
    }
  );
</script>

<style scoped>
.outline-content {
  height: 100%;
  :deep(.n-tree-node-wrapper .n-tree-node-switcher.n-tree-node-switcher--hide) {
    display: none;
  }
}

.active-chapter {
  background-color: var(--n-merged-color-hover);
  border-radius: 6px;
}
</style>
