<template>
  <n-config-provider
    :theme="nTheme"
    :theme-overrides="nTheme === null ? lightThemeOverrides : darkThemeOverrides"
  >
    <ReaderMain
      :theme="theme"
      @toggle-theme="toggleTheme"
    />
  </n-config-provider>
</template>

<script setup lang="ts">
  import { ref, onMounted, onUnmounted, computed, ComputedRef } from 'vue';
  import { NConfigProvider, darkTheme, GlobalThemeOverrides } from 'naive-ui';
  import { BuiltInGlobalTheme } from 'naive-ui/es/themes/interface';

  import ReaderMain from './views/index.vue';

  // 主题管理
  const theme = ref('default');

  const nTheme = ref<BuiltInGlobalTheme | null>(null);

  const bgColor = ref('#f3ead3');

  const darkBgColor = ref('#1a1b26');

  const toggleTheme = () => {
    theme.value = theme.value === 'default' ? 'dark' : 'default';
    nTheme.value = theme.value === 'default' ? null : darkTheme;
  };

  const toggleTransparent = () => {
    bgColor.value = bgColor.value === '#f3ead3' ? 'transparent' : '#f3ead3';
    darkBgColor.value = darkBgColor.value === '#1a1b26' ? 'transparent' : '#1a1b26';
  };

  // 添加键盘事件监听器
  const handleKeyDown = (e: KeyboardEvent) => {
    // 检测Ctrl+T组合键
    if (e.ctrlKey && e.key === 't') {
      console.log(e);

      e.preventDefault(); // 阻止浏览器默认行为（打开新标签页）
      toggleTransparent();
    }
  };

  // 在组件挂载时添加事件监听
  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  // 在组件卸载时移除事件监听，防止内存泄漏
  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  const lightThemeOverrides: ComputedRef<GlobalThemeOverrides> = computed(() => {
    return {
      common: {
        baseColor: bgColor.value,
        bodyColor: bgColor.value,
        inputColor: bgColor.value,
        primaryColor: '#8da101',
        primaryColorHover: '#9eb401',
        primaryColorPressed: '#7c9001',
        primaryColorSuppl: '#a6bc01',
        dividerColor: '#ddd8be',
        borderColor: '#ddd8be',
        textColorBase: '#5c6a72',
      },
      Button: {
        textColor: '#5c6a72',
      },
      Layout: {
        color: bgColor.value,
        headerColor: bgColor.value,
        siderColor: bgColor.value,
      },
      List: {
        color: bgColor.value,
      },
      Tree: { nodeColorHover: '#e5e6c5' },
      Popover: { color: bgColor.value, textColor: '#5c6a72' },
    };
  });

  const darkThemeOverrides: ComputedRef<GlobalThemeOverrides> = computed(() => {
    return {
      common: {
        baseColor: darkBgColor.value,
        bodyColor: darkBgColor.value,
        inputColor: darkBgColor.value,
        primaryColor: '#bb9af7',
        primaryColorHover: '#d8caf3',
        primaryColorPressed: '#ac85f3',
        primaryColorSuppl: '#c0a5f2',
        dividerColor: '#414868',
        borderColor: '#414868',
        textColorBase: '#c0caf5',
      },
      Button: {
        textColor: '#c0caf5',
      },
      Layout: {
        color: darkBgColor.value,
        headerColor: darkBgColor.value,
        siderColor: darkBgColor.value,
      },
      List: {
        color: darkBgColor.value,
      },
      Popover: { color: darkBgColor.value, textColor: '#c0caf5' },
    };
  });
</script>
