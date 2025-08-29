use base64::{engine::general_purpose, Engine as _};
use epub::doc::EpubDoc;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

// 修改现有结构，移除images字段避免重复
#[derive(Debug, Serialize, Deserialize)]
pub struct EpubChapter {
    pub title: String,
    pub content: String,
    pub html_content: String,
    // 移除 images 字段，改为按需加载
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpubImage {
    pub id: String,
    pub mime_type: String,
    pub data: String, // base64编码的图片数据
}

// 新增轻量级章节信息结构
#[derive(Debug, Serialize, Deserialize)]
pub struct EpubChapterInfo {
    pub title: String,
    pub index: usize,
    pub spine_id: String, // 用于定位章节
}

type EpubDocument = EpubDoc<BufReader<File>>;


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
                   // 不考虑扩展名的匹配
                   get_filename_without_extension(&resource_filename) == get_filename_without_extension(&src_filename) ||
                   // 包含关系匹配，不考虑扩展名
                   get_filename_without_extension(&resource_filename).contains(&get_filename_without_extension(&src_filename)) ||
                   get_filename_without_extension(&src_filename).contains(&get_filename_without_extension(&resource_filename)) ||
                   // 处理类似 x01.jpg 和 01.jpg 的情况
                   (resource_filename.len() > src_filename.len() && 
                    resource_filename.ends_with(&src_filename)) ||
                   (src_filename.len() > resource_filename.len() && 
                    src_filename.ends_with(&resource_filename)) ||
                   // 处理数字ID的情况，如 ../images/00205.jpeg
                    extract_number_id(&src_filename) == extract_number_id(&resource_filename) {
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
    
    // 按优先级排序的选择器
    let selectors = vec![
        ".chapter-title", ".title", ".chapter-name",
        "h1", "h2", "h3"
    ];
    
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let title = element.text().collect::<String>().trim().to_string();
                
                // 验证是否像章节标题
                if is_valid_chapter_title(&title) {
                    return Some(title);
                }
            }
        }
    }
    
    None
}

// 新增：验证章节标题的函数
fn is_valid_chapter_title(title: &str) -> bool {
    if title.is_empty() || title.len() > 100 {
        return false;
    }
    
    // 使用类似txt文件的正则模式验证
    let chapter_patterns = vec![
        r"第[零一二三四五六七八九十百千万0-9]+章",
        r"第[零一二三四五六七八九十百千万0-9]+回",
        r"第[零一二三四五六七八九十百千万0-9]+节",
        r"Chapter\s+[0-9]+",
    ];
    
    for pattern in chapter_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(title) {
                return true;
            }
        }
    }
    
    // 如果不匹配标准格式，但长度合理且不包含特殊字符，也可能是标题
    title.len() <= 50 && !title.contains('<') && !title.contains('>')
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

#[tauri::command]
pub async fn load_epub_file(file_path: String) -> Result<Vec<EpubChapter>, String> {
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
        });
    }

    Ok(chapters)
}

#[tauri::command]
pub async fn get_epub_info(file_path: String) -> Result<Vec<EpubChapterInfo>, String> {
    let epub_doc = match EpubDoc::new(&file_path) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Failed to open EPUB file: {}", e)),
    };

    let mut chapters_info = Vec::new();
    let spine_len = epub_doc.get_num_pages();

    for i in 0..spine_len {
        let title = format!("第{}章", i + 1);
        chapters_info.push(EpubChapterInfo {
            title,
            index: i,
            spine_id: i.to_string(),
        });
    }

    Ok(chapters_info)
}

#[tauri::command]
pub async fn load_epub_chapter(file_path: String, chapter_index: usize) -> Result<EpubChapter, String> {
    let mut epub_doc = match EpubDoc::new(&file_path) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Failed to open EPUB file: {}", e)),
    };

    epub_doc.set_current_page(chapter_index);

    let content = match epub_doc.get_current_str() {
        Some((content, _)) => content,
        None => return Err("Failed to get chapter content".to_string()),
    };

    let title = extract_title_from_html(&content)
        .unwrap_or_else(|| format!("第{}章", chapter_index + 1));

    // 只处理当前章节的图片
    let processed_html = process_chapter_images(&content, &mut epub_doc);
    let clean_content = clean_html_content(&content);

    Ok(EpubChapter {
        title,
        content: clean_content,
        html_content: processed_html,
    })
}

fn process_chapter_images(html: &str, epub_doc: &mut EpubDocument) -> String {
    // 构建当前章节需要的图片映射
    let mut images_map: HashMap<String, String> = HashMap::new();
    
    // 获取所有图片资源
    let resources = epub_doc.resources.clone();
    for (path, (_path_buf, mime_type)) in resources {
        if mime_type.starts_with("image/") {
            if let Some((image_bytes, _resource_mime)) = epub_doc.get_resource(&path) {
                let base64_data = general_purpose::STANDARD.encode(&image_bytes);
                let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                images_map.insert(path, data_url);
            }
        }
    }
    
    // 使用原来的复杂匹配逻辑
    process_html_images(html, &images_map)
}

// 修复clean_html_content函数，移除未使用的变量
fn clean_html_content(html: &str) -> String {
    let document = Html::parse_document(html);

    let body_selector = Selector::parse("body").unwrap();

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

// 提取文件名中的数字ID
fn extract_number_id(filename: &str) -> String {
    let re = Regex::new(r"(\d+)").unwrap();
    if let Some(caps) = re.captures(filename) {
        // 去除前导零
        return caps[1].trim_start_matches('0').to_string();
    }
    String::new()
}

// 获取不带扩展名的文件名
fn get_filename_without_extension(filename: &str) -> String {
    if let Some(dot_pos) = filename.rfind('.') {
        return filename[0..dot_pos].to_string();
    }
    filename.to_string()
}