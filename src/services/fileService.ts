import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export interface TxtFile {
  name: string;
  path: string;
  size: number;
}

export interface Chapter {
  title: string;
  content: string;
  html_content: string;
  start_pos: number;
  end_pos: number;
  images: Record<string, string>;
  index?: number;
  level?: number; // 章节层级
  parent_index?: number; // 父章节索引
  toc_entry?: string; // TOC条目
  detection_method?: string; // 检测方法
}

export interface EpubChapterInfo {
  title: string;
  index: number;
  spine_id: string;
  level: number;
  parent_index?: number;
  detection_method: string;
}
export enum RenderMode {
  TEXT = "text",
  HTML = "html",
}

export interface ReadingProgress {
  folder_path: string;
  current_file?: string;
  current_chapter: number;
  scroll_position: number;
  last_read_time?: string;
}

export interface AppearanceSettings {
  theme: string; // "default" 或 "dark"
  show_file_sidebar: boolean; // 是否显示文件列表
  show_outline_sidebar: boolean; // 是否显示右侧大纲列表
  outline_collapsed: boolean; // 是否折叠右侧大纲列表
}

export class FileService {
  static async selectFolder(): Promise<string | null> {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    return selected as string | null;
  }

  // 重命名方法以支持多种文件格式
  static async scanBookFiles(folderPath: string): Promise<TxtFile[]> {
    return await invoke<TxtFile[]>("scan_txt_files", { folderPath });
  }

  // 保持向后兼容
  static async scanTxtFiles(folderPath: string): Promise<TxtFile[]> {
    return this.scanBookFiles(folderPath);
  }

  static async readTxtFile(filePath: string): Promise<string> {
    return await invoke<string>("read_txt_file", { filePath });
  }

  static async parseChapters(content: string): Promise<Chapter[]> {
    return await invoke<Chapter[]>("parse_chapters", { content });
  }

  // 加载Epub文件并解析章节
  // 获取EPUB基本信息
  static async getEpubInfo(filePath: string): Promise<EpubChapterInfo[]> {
    return await invoke<EpubChapterInfo[]>("get_epub_info", { filePath });
  }

  // 按需加载单个章节
  static async loadEpubChapter(filePath: string, chapterIndex: number): Promise<Chapter> {
    return await invoke<Chapter>("load_epub_chapter", {
      filePath,
      chapterIndex,
    });
  }

  static async saveProgress(progress: ReadingProgress): Promise<void> {
    await invoke("save_reading_progress", { progress });
  }

  static async loadProgress(): Promise<ReadingProgress | null> {
    return await invoke<ReadingProgress | null>("load_reading_progress");
  }

  static async saveAppearanceSettings(settings: AppearanceSettings): Promise<void> {
    await invoke("save_appearance_settings", { settings });
  }

  static async loadAppearanceSettings(): Promise<AppearanceSettings | null> {
    return await invoke<AppearanceSettings | null>("load_appearance_settings");
  }

  static formatFileSize(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }
}
