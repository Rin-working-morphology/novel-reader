use encoding_rs::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TxtFile {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

// 扫描文件夹中的txt文件
#[tauri::command]
pub async fn scan_txt_files(folder_path: String) -> Result<Vec<TxtFile>, String> {
    let path = Path::new(&folder_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    let mut files = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_path = entry.path();
                    if file_path.is_file() {
                        if let Some(extension) = file_path.extension() {
                            let ext_str = extension.to_str().unwrap_or("").to_lowercase();
                            if ext_str == "txt" || ext_str == "epub" {
                                // 支持 epub 文件
                                if let Some(file_name) = file_path.file_name() {
                                    let metadata = fs::metadata(&file_path).unwrap_or_else(|_| {
                                        // 在某些情况下，直接获取 metadata 可能会失败，尝试另一种方式
                                        // 注意：这种回退可能不是所有平台或文件系统都完美兼容
                                        // 更好的做法是处理 fs::metadata 的 Result
                                        #[cfg(target_os = "windows")]
                                        {
                                            // Windows特定的元数据获取（如果需要）
                                            // 不过通常 std::fs::metadata 应该足够
                                            fs::metadata(&file_path).unwrap()
                                        }
                                        #[cfg(not(target_os = "windows"))]
                                        {
                                            fs::metadata(&file_path).unwrap()
                                        }
                                    });

                                    files.push(TxtFile {
                                        name: file_name.to_string_lossy().to_string(),
                                        path: file_path.to_string_lossy().to_string(),
                                        size: metadata.len(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e)),
    }

    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

// 读取txt文件内容并检测编码
#[tauri::command]
pub async fn read_txt_file(file_path: String) -> Result<String, String> {
    match fs::read(&file_path) {
        Ok(bytes) => {
            // 检测编码
            let (cow, _encoding_used, had_errors) = GBK.decode(&bytes);
            if had_errors {
                // 如果GBK解码有错误，尝试UTF-8
                match String::from_utf8(bytes.clone()) {
                    Ok(content) => Ok(content),
                    Err(_) => {
                        // 最后尝试GB18030
                        let (cow, _, _) = GB18030.decode(&bytes);
                        Ok(cow.to_string())
                    }
                }
            } else {
                Ok(cow.to_string())
            }
        }
        Err(e) => Err(format!("Failed to read file: {}", e)),
    }
}

// 解析章节
#[tauri::command]
pub async fn parse_chapters(content: String) -> Result<Vec<Chapter>, String> {
    let mut chapters = Vec::new();

    // 章节匹配模式
    let patterns = vec![
        r"第[零一二三四五六七八九十百千万0-9]+章[\s\r\n\t]*",
        r"第[零一二三四五六七八九十百千万0-9]+回[\s\r\n\t]*",
        r"第[零一二三四五六七八九十百千万0-9]+节[\s\r\n\t]*",
        r"第[零一二三四五六七八九十百千万0-9]+部[\s\r\n\t]*",
        r"第[零一二三四五六七八九十百千万0-9]+卷[\s\r\n\t]*",
        r"Chapter\s+\d+[\r\n\t ]+",
    ];

    let mut chapter_positions = Vec::new();

    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        for mat in re.find_iter(&content) {
            chapter_positions.push((mat.start(), mat.end(), mat.as_str().to_string()));
        }
    }

    // 按位置排序
    chapter_positions.sort_by_key(|&(start, _, _)| start);

    // 去重（保留第一个匹配的）
    chapter_positions.dedup_by_key(|&mut (start, _, _)| start);

    if chapter_positions.is_empty() {
        // 如果没有找到章节，将整个文件作为一章
        chapters.push(Chapter {
            title: "全文".to_string(),
            content: content.clone(),
            start_pos: 0,
            end_pos: content.len(),
        });
    } else {
        for (i, (start, end, title)) in chapter_positions.iter().enumerate() {
            let chapter_start = *start;
            let chapter_end = if i + 1 < chapter_positions.len() {
                chapter_positions[i + 1].0
            } else {
                content.len()
            };

            let chapter_content = content[chapter_start..chapter_end].to_string();

            chapters.push(Chapter {
                title: title.trim().to_string(),
                content: chapter_content,
                start_pos: chapter_start,
                end_pos: chapter_end,
            });
        }
    }

    Ok(chapters)
}