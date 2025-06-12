import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface TxtFile {
  name: string;
  path: string;
  size: number;
}

export interface Chapter {
  title: string;
  content: string;
  start_pos: number;
  end_pos: number;
  html_content: string; // 包含图片的HTML内容
  images: Record<string, string>; // 图片映射
}

// 添加渲染模式枚举
export enum RenderMode {
  TEXT = 'text',
  HTML = 'html',
}

export interface ReadingProgress {
  folder_path: string;
  current_file: string;
  current_chapter: number;
  scroll_position: number;
  last_read_time: string;
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
    return await invoke<TxtFile[]>('scan_txt_files', { folderPath });
  }

  // 保持向后兼容
  static async scanTxtFiles(folderPath: string): Promise<TxtFile[]> {
    return this.scanBookFiles(folderPath);
  }

  static async readTxtFile(filePath: string): Promise<string> {
    return await invoke<string>('read_txt_file', { filePath });
  }

  static async parseChapters(content: string): Promise<Chapter[]> {
    return await invoke<Chapter[]>('parse_chapters', { content });
  }

  // 新增方法：加载Epub文件并解析章节
  static async loadEpubFile(filePath: string): Promise<Chapter[]> {
    return await invoke<Chapter[]>('load_epub_file', { filePath });
  }

  static async saveProgress(progress: ReadingProgress): Promise<void> {
    await invoke('save_reading_progress', { progress });
  }

  static async loadProgress(): Promise<ReadingProgress | null> {
    return await invoke<ReadingProgress | null>('load_reading_progress');
  }

  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
}
