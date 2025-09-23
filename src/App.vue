<template>
  <n-config-provider
    :theme="nTheme"
    :theme-overrides="nTheme === null ? lightThemeOverrides : darkThemeOverrides"
  >
    <ReaderMain
      :theme="theme"
      :sidebar-collapsed="sidebarCollapsed"
      :outline-visible="outlineVisible"
      :outline-collapsed="appearanceSettings.outline_collapsed"
      @toggle-theme="toggleTheme"
      @toggle-sidebar="handleToggleSidebar"
      @toggle-outline-visible="handleToggleOutlineVisible"
      @toggle-outline-collapse="handleToggleOutlineCollapse"
    />
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, ComputedRef, watch } from "vue";
import { NConfigProvider, darkTheme, GlobalThemeOverrides } from "naive-ui";
import { BuiltInGlobalTheme } from "naive-ui/es/themes/interface";
import { restoreStateCurrent, StateFlags } from "@tauri-apps/plugin-window-state";

import ReaderMain from "./views/index.vue";
import { FileService, type AppearanceSettings } from "./services/fileService";

// 外观设置状态管理
const appearanceSettings = ref<AppearanceSettings>({
  theme: "default",
  show_file_sidebar: true,
  show_outline_sidebar: false,
  outline_collapsed: false,
});

// 主题管理
const theme = ref("default");
const nTheme = ref<BuiltInGlobalTheme | null>(null);
const bgColor = ref("#f3ead3");
const darkBgColor = ref("#1a1b26");

// 界面状态
const sidebarCollapsed = ref(false);
const outlineVisible = ref(false);

const toggleTheme = () => {
  theme.value = theme.value === "default" ? "dark" : "default";
  nTheme.value = theme.value === "default" ? null : darkTheme;
  appearanceSettings.value.theme = theme.value;
};

// 保存外观设置
const saveAppearanceSettings = async () => {
  try {
    await FileService.saveAppearanceSettings(appearanceSettings.value);
  } catch (error) {
    console.error("Failed to save appearance settings:", error);
  }
};

// 加载外观设置
const loadAppearanceSettings = async () => {
  try {
    const settings = await FileService.loadAppearanceSettings();
    if (settings) {
      appearanceSettings.value = settings;
      theme.value = settings.theme;
      nTheme.value = settings.theme === "default" ? null : darkTheme;
      sidebarCollapsed.value = settings.show_file_sidebar;
      outlineVisible.value = settings.show_outline_sidebar;
    }
  } catch (error) {
    console.error("Failed to load appearance settings:", error);
  }
};

const toggleTransparent = () => {
  bgColor.value = bgColor.value === "#f3ead3" ? "transparent" : "#f3ead3";
  darkBgColor.value = darkBgColor.value === "#1a1b26" ? "transparent" : "#1a1b26";
};

// 添加键盘事件监听器
const handleKeyDown = (e: KeyboardEvent) => {
  // 检测Ctrl+T组合键
  if (e.ctrlKey && e.key === "t") {
    e.preventDefault(); // 阻止浏览器默认行为（打开新标签页）
    toggleTransparent();
  }
};

// 处理侧边栏和大纲切换
const handleToggleSidebar = (collapsed: boolean) => {
  sidebarCollapsed.value = collapsed;
  appearanceSettings.value.show_file_sidebar = collapsed;
};

const handleToggleOutlineVisible = () => {
  outlineVisible.value = !outlineVisible.value;
  appearanceSettings.value.show_outline_sidebar = outlineVisible.value;
};

const handleToggleOutlineCollapse = (collapsed: boolean) => {
  appearanceSettings.value.outline_collapsed = collapsed;
};

// 在组件挂载时添加事件监听
onMounted(async () => {
  restoreStateCurrent(StateFlags.ALL);
  window.addEventListener("keydown", handleKeyDown);
  await loadAppearanceSettings();
});

// 在组件卸载时移除事件监听，防止内存泄漏
onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});

// 监听外观设置变化并自动保存
watch(
  () => appearanceSettings.value,
  () => {
    saveAppearanceSettings();
  },
  { deep: true }
);

const lightThemeOverrides: ComputedRef<GlobalThemeOverrides> = computed(() => {
  return {
    common: {
      baseColor: bgColor.value,
      bodyColor: bgColor.value,
      inputColor: bgColor.value,
      popoverColor: bgColor.value,
      primaryColor: "#8da101",
      primaryColorHover: "#9eb401",
      primaryColorPressed: "#7c9001",
      primaryColorSuppl: "#a6bc01",
      dividerColor: "#ddd8be",
      borderColor: "#ddd8be",
      hoverColor: "#e5e6c5",
      textColorBase: "#5c6a72",
    },
    Button: {
      textColor: "#5c6a72",
    },
    Layout: {
      color: bgColor.value,
      headerColor: bgColor.value,
      siderColor: bgColor.value,
    },
    List: {
      color: bgColor.value,
    },
    Tree: { nodeColorHover: "#e5e6c5" },
    Popover: { color: bgColor.value, textColor: "#5c6a72" },
  };
});

const darkThemeOverrides: ComputedRef<GlobalThemeOverrides> = computed(() => {
  return {
    common: {
      baseColor: darkBgColor.value,
      bodyColor: darkBgColor.value,
      inputColor: darkBgColor.value,
      popoverColor: darkBgColor.value,
      primaryColor: "#bb9af7",
      primaryColorHover: "#d8caf3",
      primaryColorPressed: "#ac85f3",
      primaryColorSuppl: "#c0a5f2",
      dividerColor: "#414868",
      borderColor: "#414868",
      textColorBase: "#c0caf5",
    },
    Button: {
      textColor: "#c0caf5",
    },
    Layout: {
      color: darkBgColor.value,
      headerColor: darkBgColor.value,
      siderColor: darkBgColor.value,
    },
    List: {
      color: darkBgColor.value,
    },
    Popover: { color: darkBgColor.value, textColor: "#c0caf5" },
  };
});
</script>
