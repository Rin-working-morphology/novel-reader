<template>
  <n-layout-sider
    bordered
    collapse-mode="width"
    :collapsed-width="0"
    :width="280"
    show-trigger="bar"
    content-style="padding: 16px;"
  >
    <div class="sidebar-content">
      <div class="folder-section">
        <n-button
          type="primary"
          block
          ghost
          @click="handleSelectFolder"
          :loading="loading"
        >
          选择文件夹
        </n-button>
        <n-text
          v-if="currentFolder"
          depth="3"
          style="margin-top: 8px; font-size: 12px"
        >
          {{ currentFolder }}
        </n-text>
      </div>

      <n-divider style="margin: 16px 0" />

      <div class="file-list">
        <n-scrollbar style="max-height: calc(100vh - 200px)">
          <n-list>
            <n-list-item
              v-for="file in txtFiles"
              :key="file.path"
              @click="$emit('select-file', file)"
              :class="{ 'active-file': currentFile?.path === file.path }"
              style="cursor: pointer; padding: 8px 12px"
            >
              <div class="file-item">
                <n-text>{{ file.name }}</n-text>
                <n-text
                  depth="3"
                  style="font-size: 12px"
                >
                  {{ FileService.formatFileSize(file.size) }}
                </n-text>
              </div>
            </n-list-item>
          </n-list>
        </n-scrollbar>
      </div>
    </div>
  </n-layout-sider>
</template>

<script setup lang="ts">
  import { ref } from 'vue';
  import { NLayoutSider, NButton, NText, NDivider, NScrollbar, NList, NListItem, useMessage } from 'naive-ui';
  import { FileService, type TxtFile } from '../services/fileService';

  interface Props {
    collapsed: boolean;
    currentFolder: string;
    txtFiles: TxtFile[];
    currentFile: TxtFile | null;
  }

  defineProps<Props>();
  const emit = defineEmits<{
    'select-file': [file: TxtFile];
    'folder-selected': [folderPath: string, files: TxtFile[]];
  }>();

  const message = useMessage();
  const loading = ref(false);

  const handleSelectFolder = async () => {
    try {
      const selected = await FileService.selectFolder();
      if (selected) {
        loading.value = true;
        const files = await FileService.scanBookFiles(selected); // 使用新的方法名
        emit('folder-selected', selected, files);
        message.success(`找到 ${files.length} 个文件（TXT/EPUB）`);
      }
    } catch (error) {
      message.error('选择文件夹失败: ' + error);
    } finally {
      loading.value = false;
    }
  };
</script>

<style scoped>
  .sidebar-content {
    height: 100%;
  }

  .folder-section {
    margin-bottom: 16px;
  }

  .file-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .active-file {
    background-color: var(--n-item-color-hover);
    border-radius: 6px;
  }
</style>
