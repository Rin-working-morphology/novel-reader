
use base64::{engine::general_purpose, Engine as _};
use encoding_rs::*;
use epub::doc::EpubDoc;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct TxtFile {
    name: String,
    path: String,
    size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chapter {
    title: String,
    content: String,
    start_pos: usize,
    end_pos: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ReadingProgress {
    folder_path: String,
    current_file: String,
    current_chapter: usize,
    scroll_position: f64,
    last_read_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EpubChapter {
    title: String,
    content: String,
    html_content: String,            // 保留原始HTML
    images: HashMap<String, String>, // 图片路径 -> base64数据
}

#[derive(Debug, Serialize, Deserialize)]
struct EpubImage {
    id: String,
    mime_type: String,
    data: String, // base64编码的图片数据
}

// 扫描文件夹中的txt文件
#[tauri::command]
async fn scan_txt_files(folder_path: String) -> Result<Vec<TxtFile>, String> {
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
async fn read_txt_file(file_path: String) -> Result<String, String> {
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
async fn parse_chapters(content: String) -> Result<Vec<Chapter>, String> {
    let mut chapters = Vec::new();

    // 章节匹配模式
    let patterns = vec![
        r"第[一二三四五六七八九十百千万\d]+章[\r\n\t ]+",
        r"第[一二三四五六七八九十百千万\d]+回[\r\n\t ]+",
        r"第[一二三四五六七八九十百千万\d]+节[\r\n\t ]+",
        r"第[一二三四五六七八九十百千万\d]+部[\r\n\t ]+",
        r"第[一二三四五六七八九十百千万\d]+卷[\r\n\t ]+",
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

// 保存阅读进度
#[tauri::command]
async fn save_reading_progress(
    app_handle: tauri::AppHandle,
    progress: ReadingProgress,
) -> Result<(), String> {
    let store = tauri_plugin_store::StoreBuilder::new(&app_handle, "reading_progress.json").build();

    match store {
        Ok(store) => {
            store.set("progress", serde_json::to_value(progress).unwrap());

            if let Err(e) = store.save() {
                return Err(format!("Failed to save store: {}", e));
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to create store: {}", e)),
    }
}

// 加载阅读进度
#[tauri::command]
async fn load_reading_progress(
    app_handle: tauri::AppHandle,
) -> Result<Option<ReadingProgress>, String> {
    let store = tauri_plugin_store::StoreBuilder::new(&app_handle, "reading_progress.json").build();

    match store {
        Ok(store) => {
            if let Err(e) = store.reload() {
                return Ok(None); // 文件不存在时返回None
            }

            match store.get("progress") {
                Some(value) => match serde_json::from_value::<ReadingProgress>(value.clone()) {
                    Ok(progress) => Ok(Some(progress)),
                    Err(_) => Ok(None),
                },
                None => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

//处理HTML中的图片引用
fn process_html_images(html: &str, images_map: &HashMap<String, String>) -> String {
    let mut processed_html = html.to_string();

    // 使用正则表达式替换图片src属性
    let img_regex = Regex::new(r#"<img[^>]*src=["']([^"']*)["'][^>]*>"#).unwrap();

    processed_html = img_regex
        .replace_all(&processed_html, |caps: &regex::Captures| {
            let original_src = &caps[1];
            
            // 提取文件名（不包含路径）
            let src_filename = original_src
                .split('/')
                .last()
                .unwrap_or(original_src)
                .to_lowercase();

            // 尝试在images_map中找到对应的图片
            for (path, data_url) in images_map {
                // 提取资源ID的文件名部分
                let resource_filename = path
                    .split('/')
                    .last()
                    .unwrap_or(path)
                    .to_lowercase();
                
                // 多种匹配方式
                if resource_filename == src_filename ||
                   resource_filename.contains(&src_filename.replace(".jpg", "")) ||
                   src_filename.contains(&resource_filename.replace(".jpg", "")) ||
                   // 处理类似 x01.jpg 和 01.jpg 的情况
                   (resource_filename.len() > src_filename.len() && 
                    resource_filename.ends_with(&src_filename)) ||
                   (src_filename.len() > resource_filename.len() && 
                    src_filename.ends_with(&resource_filename)) {
                    
                    println!("Successfully matched: '{}' -> '{}'", original_src, path);
                    return caps[0].replace(original_src, data_url);
                }
            }

            println!("No match found for: '{}'", original_src);
            // 如果找不到，保持原样
            caps[0].to_string()
        })
        .to_string();

    processed_html
}

// 改进的HTML清理函数，保留基本格式
fn clean_html_content_preserve_format(html: &str) -> String {
    let document = Html::parse_document(html);
    let mut content = String::new();

    // 提取body内容
    let body_selector = Selector::parse("body").unwrap();
    let container = document
        .select(&body_selector)
        .next()
        .unwrap_or_else(|| document.root_element());

    extract_formatted_text(container, &mut content);

    // 清理多余的空白字符但保留段落结构
    let re = Regex::new(r"\n\s*\n\s*\n").unwrap();
    re.replace_all(&content, "\n\n").trim().to_string()
}

fn extract_formatted_text(element: scraper::ElementRef, content: &mut String) {
    for node in element.children() {
        match node.value() {
            scraper::Node::Text(text) => {
                content.push_str(&text.text);
            }
            scraper::Node::Element(elem) => match elem.name() {
                "script" | "style" => continue,
                "img" => {
                    content.push_str("[图片]");
                }
                "br" => {
                    content.push('\n');
                }
                "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    content.push('\n');
                    if let Some(child_element) = scraper::ElementRef::wrap(node) {
                        extract_formatted_text(child_element, content);
                    }
                    content.push('\n');
                }
                _ => {
                    if let Some(child_element) = scraper::ElementRef::wrap(node) {
                        extract_formatted_text(child_element, content);
                    }
                }
            },
            _ => {}
        }
    }
}

// 从HTML中提取标题
fn extract_title_from_html(html: &str) -> Option<String> {
    let document = Html::parse_document(html);

    // 尝试多种标题选择器
    let selectors = vec!["h1", "h2", "h3", "title", ".chapter-title", ".title"];

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let title = element.text().collect::<String>().trim().to_string();
                if !title.is_empty() {
                    return Some(title);
                }
            }
        }
    }

    None
}

fn is_block_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "br" | "hr" | "blockquote" | "pre"
    )
}

fn extract_text_content(element: scraper::ElementRef, content: &mut String) {
    for node in element.children() {
        match node.value() {
            scraper::Node::Text(text) => {
                content.push_str(&text.text);
            }
            scraper::Node::Element(elem) => {
                // 跳过script和style标签
                if elem.name() != "script" && elem.name() != "style" {
                    if let Some(child_element) = scraper::ElementRef::wrap(node) {
                        // 在块级元素前后添加换行
                        if is_block_element(elem.name()) {
                            content.push('\n');
                        }
                        extract_text_content(child_element, content);
                        if is_block_element(elem.name()) {
                            content.push('\n');
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// 清理HTML内容，提取纯文本
fn clean_html_content(html: &str) -> String {
    let document = Html::parse_document(html);

    // 移除script和style标签
    let body_selector = Selector::parse("body").unwrap();
    let script_selector = Selector::parse("script").unwrap();
    let style_selector = Selector::parse("style").unwrap();

    let mut content = String::new();

    // 提取body内容，如果没有body则提取整个文档
    let container = document
        .select(&body_selector)
        .next()
        .unwrap_or_else(|| document.root_element());

    // 递归提取文本内容
    extract_text_content(container, &mut content);

    // 清理多余的空白字符
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&content, " ").trim().to_string()
}

#[tauri::command]
async fn load_epub_file(file_path: String) -> Result<Vec<EpubChapter>, String> {
    let mut epub_doc = match EpubDoc::new(&file_path) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Failed to open EPUB file: {}", e)),
    };

    let mut chapters = Vec::new();

    // 首先提取所有图片资源
    let mut images_map: HashMap<String, String> = HashMap::new();

    // 获取所有资源
    let resources = epub_doc.resources.clone();
    for (path, (_path_buf, mime_type)) in resources {
        if mime_type.starts_with("image/") {
            println!("path: {}", path);
            // 使用get_resource方法获取实际的图片数据
            if let Some((image_bytes, _resource_mime)) = epub_doc.get_resource(&path) {
                // 将图片转换为base64
                let base64_data = general_purpose::STANDARD.encode(&image_bytes);
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                images_map.insert(path, data_url);
            }
        }
    }

    // 获取所有spine项目（章节顺序）
    let spine_len = epub_doc.get_num_pages();

    for i in 0..spine_len {
        epub_doc.set_current_page(i);

        // 获取当前页面内容
        let content = match epub_doc.get_current_str() {
            Some((content, _)) => content,
            None => continue,
        };

        // 获取章节标题
        let title = extract_title_from_html(&content).unwrap_or_else(|| format!("第{}章", i + 1));

        // 处理HTML内容中的图片引用
        let processed_html = process_html_images(&content, &images_map);

        // 提取纯文本（作为备用）
        let clean_content = clean_html_content(&content);

        chapters.push(EpubChapter {
            title,
            content: clean_content,
            html_content: processed_html,
            images: images_map.clone(),
        });
    }

    Ok(chapters)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_txt_files,
            read_txt_file,
            parse_chapters,
            load_epub_file, // 添加这行
            save_reading_progress,
            load_reading_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
