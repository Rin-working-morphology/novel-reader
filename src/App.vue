<template>
  <n-config-provider
    :theme="nTheme"
    :theme-overrides="nTheme === null ? lightThemeOverrides : darkThemeOverrides"
  >
    <ReaderMain
      :theme="theme"
      :sidebar-collapsed="sidebarCollapsed"
      :outline-visible="outlineVisible"
      @toggle-theme="toggleTheme"
      @toggle-sidebar="handleToggleSidebar"
      @toggle-outline-visible="handleToggleOutlineVisible"
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
});

// 主题管理
const theme = ref("default");
const nTheme = ref<BuiltInGlobalTheme | null>(null);
const lightBgColor = "#f3ead3";
const darkDefaultBgColor = "#1a1b26";
const transparentColor = "transparent";
const transparentBorder = "1px solid transparent";

const bgColor = ref(lightBgColor);
const darkBgColor = ref(darkDefaultBgColor);
const uiFontFamily = '"LXGW Neo XiHei", system-ui, sans-serif';

// 界面状态
const sidebarCollapsed = ref(false);
const outlineVisible = ref(false);
const sidebarCollapsedBeforeTransparent = ref<boolean | null>(null);

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
      appearanceSettings.value = {
        theme: settings.theme,
        show_file_sidebar: settings.show_file_sidebar,
        show_outline_sidebar: settings.show_outline_sidebar,
      };
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
  const enteringTransparent = bgColor.value !== transparentColor;

  bgColor.value = enteringTransparent ? transparentColor : lightBgColor;
  darkBgColor.value = enteringTransparent ? transparentColor : darkDefaultBgColor;

  if (enteringTransparent) {
    sidebarCollapsedBeforeTransparent.value = sidebarCollapsed.value;
    sidebarCollapsed.value = true;
  } else if (sidebarCollapsedBeforeTransparent.value !== null) {
    sidebarCollapsed.value = sidebarCollapsedBeforeTransparent.value;
    sidebarCollapsedBeforeTransparent.value = null;
  }

  appearanceSettings.value.show_file_sidebar = sidebarCollapsed.value;
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

const handleToggleOutlineVisible = (visible: boolean) => {
  outlineVisible.value = visible;
  appearanceSettings.value.show_outline_sidebar = outlineVisible.value;
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
  const isTransparent = bgColor.value === transparentColor;
  const primaryColor = "#8da101";
  const primaryColorHover = "#9eb401";
  const primaryColorPressed = "#7c9001";
  const primaryColorSuppl = "#a6bc01";
  const accentChromeColor = isTransparent ? transparentColor : primaryColor;
  const lineColor = isTransparent ? transparentColor : "#ddd8be";
  const hoverColor = isTransparent ? transparentColor : "#e5e6c5";
  const textColor = "#5c6a72";

  return {
    common: {
      fontFamily: uiFontFamily,
      baseColor: bgColor.value,
      bodyColor: bgColor.value,
      inputColor: bgColor.value,
      popoverColor: bgColor.value,
      cardColor: bgColor.value,
      primaryColor,
      primaryColorHover,
      primaryColorPressed,
      primaryColorSuppl,
      dividerColor: lineColor,
      borderColor: lineColor,
      hoverColor,
      textColorBase: textColor,
    },
    Button: {
      textColor,
      ...(isTransparent
        ? {
            border: transparentBorder,
            borderHover: transparentBorder,
            borderPressed: transparentBorder,
            borderFocus: transparentBorder,
            borderPrimary: transparentBorder,
            borderHoverPrimary: transparentBorder,
            borderPressedPrimary: transparentBorder,
            borderFocusPrimary: transparentBorder,
            textColorHover: textColor,
            textColorPressed: textColor,
            textColorFocus: textColor,
            textColorTextHover: textColor,
            textColorTextPressed: textColor,
            textColorTextFocus: textColor,
            textColorGhostPrimary: textColor,
            textColorGhostHoverPrimary: textColor,
            textColorGhostPressedPrimary: textColor,
            textColorGhostFocusPrimary: textColor,
          }
        : {}),
    },
    Divider: { color: lineColor },
    GradientText: {
      colorStartPrimary: accentChromeColor,
      colorEndPrimary: accentChromeColor,
    },
    Layout: {
      color: bgColor.value,
      headerColor: bgColor.value,
      siderColor: bgColor.value,
      headerBorderColor: lineColor,
      siderBorderColor: lineColor,
      siderToggleButtonBorder: isTransparent ? transparentBorder : `1px solid ${lineColor}`,
      siderToggleButtonColor: bgColor.value,
      siderToggleBarColor: lineColor,
      siderToggleBarColorHover: lineColor,
    },
    List: {
      color: bgColor.value,
      colorHover: hoverColor,
      borderColor: lineColor,
    },
    Tree: {
      nodeColorHover: hoverColor,
      nodeColorPressed: hoverColor,
      nodeColorActive: hoverColor,
      lineColor,
    },
    Radio: {
      ...(isTransparent
        ? {
            boxShadow: `inset 0 0 0 1px ${lineColor}`,
            boxShadowActive: `inset 0 0 0 1px ${lineColor}`,
            boxShadowFocus: `inset 0 0 0 1px ${lineColor}`,
            boxShadowHover: `inset 0 0 0 1px ${lineColor}`,
            colorActive: transparentColor,
            dotColorActive: transparentColor,
            buttonBorderColor: lineColor,
            buttonBorderColorActive: lineColor,
            buttonBorderColorHover: lineColor,
          }
        : {}),
    },
    InternalSelection: {
      ...(isTransparent
        ? {
            color: transparentColor,
            colorActive: transparentColor,
            border: transparentBorder,
            borderHover: transparentBorder,
            borderActive: transparentBorder,
            borderFocus: transparentBorder,
            boxShadowHover: "none",
            boxShadowActive: "none",
            boxShadowFocus: "none",
          }
        : {}),
    },
    Popover: { color: bgColor.value, textColor },
  };
});

const darkThemeOverrides: ComputedRef<GlobalThemeOverrides> = computed(() => {
  const isTransparent = darkBgColor.value === transparentColor;
  const primaryColor = "#bb9af7";
  const primaryColorHover = "#d8caf3";
  const primaryColorPressed = "#ac85f3";
  const primaryColorSuppl = "#c0a5f2";
  const accentChromeColor = isTransparent ? transparentColor : primaryColor;
  const lineColor = isTransparent ? transparentColor : "#414868";
  const hoverColor = isTransparent ? transparentColor : "#2f354f";
  const textColor = "#c0caf5";

  return {
    common: {
      fontFamily: uiFontFamily,
      baseColor: darkBgColor.value,
      bodyColor: darkBgColor.value,
      inputColor: darkBgColor.value,
      popoverColor: darkBgColor.value,
      cardColor: darkBgColor.value,
      primaryColor,
      primaryColorHover,
      primaryColorPressed,
      primaryColorSuppl,
      dividerColor: lineColor,
      borderColor: lineColor,
      hoverColor,
      textColorBase: textColor,
    },
    Button: {
      textColor,
      ...(isTransparent
        ? {
            border: transparentBorder,
            borderHover: transparentBorder,
            borderPressed: transparentBorder,
            borderFocus: transparentBorder,
            borderPrimary: transparentBorder,
            borderHoverPrimary: transparentBorder,
            borderPressedPrimary: transparentBorder,
            borderFocusPrimary: transparentBorder,
            textColorHover: textColor,
            textColorPressed: textColor,
            textColorFocus: textColor,
            textColorTextHover: textColor,
            textColorTextPressed: textColor,
            textColorTextFocus: textColor,
            textColorGhostPrimary: textColor,
            textColorGhostHoverPrimary: textColor,
            textColorGhostPressedPrimary: textColor,
            textColorGhostFocusPrimary: textColor,
          }
        : {}),
    },
    Divider: { color: lineColor },
    GradientText: {
      colorStartPrimary: accentChromeColor,
      colorEndPrimary: accentChromeColor,
    },
    Layout: {
      color: darkBgColor.value,
      headerColor: darkBgColor.value,
      siderColor: darkBgColor.value,
      headerBorderColor: lineColor,
      siderBorderColor: lineColor,
      siderToggleButtonBorder: isTransparent ? transparentBorder : `1px solid ${lineColor}`,
      siderToggleButtonColor: darkBgColor.value,
      siderToggleBarColor: lineColor,
      siderToggleBarColorHover: lineColor,
    },
    List: {
      color: darkBgColor.value,
      colorHover: hoverColor,
      borderColor: lineColor,
    },
    Tree: {
      nodeColorHover: hoverColor,
      nodeColorPressed: hoverColor,
      nodeColorActive: hoverColor,
      lineColor,
    },
    Radio: {
      ...(isTransparent
        ? {
            boxShadow: `inset 0 0 0 1px ${lineColor}`,
            boxShadowActive: `inset 0 0 0 1px ${lineColor}`,
            boxShadowFocus: `inset 0 0 0 1px ${lineColor}`,
            boxShadowHover: `inset 0 0 0 1px ${lineColor}`,
            colorActive: transparentColor,
            dotColorActive: transparentColor,
            buttonBorderColor: lineColor,
            buttonBorderColorActive: lineColor,
            buttonBorderColorHover: lineColor,
          }
        : {}),
    },
    InternalSelection: {
      ...(isTransparent
        ? {
            color: transparentColor,
            colorActive: transparentColor,
            border: transparentBorder,
            borderHover: transparentBorder,
            borderActive: transparentBorder,
            borderFocus: transparentBorder,
            boxShadowHover: "none",
            boxShadowActive: "none",
            boxShadowFocus: "none",
          }
        : {}),
    },
    Popover: { color: darkBgColor.value, textColor },
  };
});
</script>
