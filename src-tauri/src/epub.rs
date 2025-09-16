use base64::{engine::general_purpose, Engine as _};
use epub::doc::EpubDoc;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

// 增强的章节结构
#[derive(Debug, Serialize, Deserialize)]
pub struct EpubChapter {
    pub title: String,
    pub content: String,
    pub html_content: String,
    pub level: u8,                   // 章节层级 (1=章, 2=节, 3=小节)
    pub parent_index: Option<usize>, // 父章节索引
    pub toc_entry: Option<String>,   // TOC中的原始条目
    pub detection_method: String,    // 检测方法
}

// 轻量级章节信息结构
#[derive(Debug, Serialize, Deserialize)]
pub struct EpubChapterInfo {
    pub title: String,
    pub index: usize,
    pub spine_id: String,
    pub level: u8,
    pub parent_index: Option<usize>,
    pub detection_method: String,
}

type EpubDocument = EpubDoc<BufReader<File>>;

// TOC条目结构
#[derive(Debug, Clone)]
struct TocEntry {
    title: String,
    href: String,
    level: u8,
    play_order: Option<u32>,
}

// 层级趋势枚举
#[derive(Debug, PartialEq)]
enum LevelTrend {
    Increasing, // 层级递增
    Decreasing, // 层级递减
    Stable,     // 层级稳定
}

// 处理HTML中的图片引用
fn process_html_images(html: &str, images_map: &HashMap<String, String>) -> String {
    let mut processed_html = html.to_string();
    let img_regex = Regex::new(r#"<img[^>]*src=["']([^"']*)["'][^>]*>"#).unwrap();

    processed_html = img_regex
        .replace_all(&processed_html, |caps: &regex::Captures| {
            let original_src = &caps[1];
            let src_filename = original_src
                .split('/')
                .last()
                .unwrap_or(original_src)
                .to_lowercase();

            for (path, data_url) in images_map {
                let resource_filename = path.split('/').last().unwrap_or(path).to_lowercase();

                if resource_filename == src_filename
                    || get_filename_without_extension(&resource_filename)
                        == get_filename_without_extension(&src_filename)
                    || extract_number_id(&src_filename) == extract_number_id(&resource_filename)
                {
                    return caps[0].replace(original_src, data_url);
                }
            }
            caps[0].to_string()
        })
        .to_string();

    processed_html
}

// 增强的标题提取函数
fn extract_title_with_confidence(html: &str, toc_entries: &[TocEntry]) -> (String, f32) {
    let document = Html::parse_document(html);
    let mut candidates = Vec::new();

    // 1. 优先级最高：从内容中提取可能的标题，然后与 TOC 匹配
    let text_content = extract_text_content_simple(&document);
    let lines: Vec<&str> = text_content.lines().collect();

    // 检查前几行是否匹配 TOC 标题
    for line in lines.iter().take(5) {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.len() < 100 {
            for toc_entry in toc_entries {
                // 精确匹配或包含匹配
                if trimmed == toc_entry.title
                    || (trimmed.contains(&toc_entry.title) && toc_entry.title.len() > 0)
                {
                    return (toc_entry.title.clone(), 1.0);
                }
            }
        }
    }
    // 2. HTML 标题标签检测
    let heading_selectors = vec![
        ("h1", 0.9),
        ("h2", 0.8),
        ("h3", 0.7),
        (".chapter-title", 0.85),
        (".title", 0.75),
    ];

    for (selector_str, confidence) in heading_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let title = element.text().collect::<String>().trim().to_string();
                if is_valid_chapter_title_enhanced(&title) {
                    candidates.push((title, confidence));
                }
            }
        }
    }

    // 3. 基于内容位置的启发式检测
    let text_content = extract_text_content_simple(&document);
    if let Some(title) = detect_title_from_content(&text_content) {
        candidates.push((title, 0.6));
    }

    // 返回置信度最高的候选
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    candidates
        .first()
        .cloned()
        .unwrap_or(("未知章节".to_string(), 0.0))
}

// 增强的标题验证
fn is_valid_chapter_title_enhanced(title: &str) -> bool {
    if title.is_empty() || title.len() > 200 {
        return false;
    }

    let chapter_patterns = vec![
        r"第[零一二三四五六七八九十百千万0-9]+[章回节部卷篇部分]",
        r"Chapter\s+[0-9IVX]+",
        r"CHAPTER\s+[0-9IVX]+",
        r"[0-9]+\.[0-9]*\s*",
        r"[IVX]+\.[\s]*",
        r"序章|楔子|引子|尾声|后记|前言|序言|目录|自序",
        r"Prologue|Epilogue|Preface|Introduction",
    ];

    for pattern in chapter_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(title) {
                return true;
            }
        }
    }

    title.len() <= 100
        && !title.contains('<')
        && !title.contains('>')
        && !title.contains("http")
        && title.chars().any(|c| c.is_alphabetic() || c.is_numeric())
}

// 构建spine到TOC的映射关系
fn build_spine_toc_mapping(
    epub_doc: &mut EpubDocument,
    toc_entries: &[TocEntry],
) -> Result<HashMap<usize, TocEntry>, String> {
    let mut spine_to_toc = HashMap::new();

    for toc_entry in toc_entries {
        if let Some(spine_index) = find_spine_index_by_href(epub_doc, &toc_entry.href) {
            spine_to_toc.insert(spine_index, toc_entry.clone());
        }
    }

    Ok(spine_to_toc)
}

// 确定章节层级和父章节关系
fn determine_chapter_hierarchy(
    current_spine_index: usize,
    spine_to_toc: &HashMap<usize, TocEntry>,
    existing_chapters: &[EpubChapter],
) -> (u8, Option<usize>) {
    if let Some(toc_entry) = spine_to_toc.get(&current_spine_index) {
        let level = toc_entry.level;
        let parent_index = find_parent_chapter_index(level, existing_chapters);
        return (level, parent_index);
    }

    let default_level = determine_default_level(current_spine_index, existing_chapters);
    let parent_index = find_parent_chapter_index(default_level, existing_chapters);

    (default_level, parent_index)
}

// 查找父章节索引
fn find_parent_chapter_index(
    current_level: u8,
    existing_chapters: &[EpubChapter],
) -> Option<usize> {
    if current_level <= 1 {
        return None;
    }

    for (index, chapter) in existing_chapters.iter().enumerate().rev() {
        if chapter.level < current_level {
            return Some(index);
        }
    }

    None
}

// 确定默认层级
fn determine_default_level(_current_spine_index: usize, existing_chapters: &[EpubChapter]) -> u8 {
    if existing_chapters.is_empty() {
        return 1;
    }

    let recent_levels: Vec<u8> = existing_chapters
        .iter()
        .rev()
        .take(3)
        .map(|ch| ch.level)
        .collect();

    if let Some(&last_level) = recent_levels.first() {
        if recent_levels.iter().all(|&level| level == last_level) {
            return last_level;
        }

        if recent_levels.len() >= 2 {
            let trend = analyze_level_trend(&recent_levels);
            match trend {
                LevelTrend::Increasing => std::cmp::min(last_level + 1, 3),
                LevelTrend::Decreasing => std::cmp::max(last_level.saturating_sub(1), 1),
                LevelTrend::Stable => last_level,
            }
        } else {
            last_level
        }
    } else {
        1
    }
}

// 分析层级趋势
fn analyze_level_trend(levels: &[u8]) -> LevelTrend {
    if levels.len() < 2 {
        return LevelTrend::Stable;
    }

    let mut increasing = 0;
    let mut decreasing = 0;

    for i in 1..levels.len() {
        if levels[i] > levels[i - 1] {
            increasing += 1;
        } else if levels[i] < levels[i - 1] {
            decreasing += 1;
        }
    }

    if increasing > decreasing {
        LevelTrend::Increasing
    } else if decreasing > increasing {
        LevelTrend::Decreasing
    } else {
        LevelTrend::Stable
    }
}

// 改进的spine索引查找函数
fn find_spine_index_by_href(epub_doc: &EpubDocument, href: &str) -> Option<usize> {
    let clean_href: &str = href
        .split('#')
        .next()
        .unwrap_or(href)
        .split('?')
        .next()
        .unwrap_or(href);

    for (spine_index, spine_item) in epub_doc.spine.iter().enumerate() {
        if spine_item.idref == clean_href {
            return Some(spine_index);
        }

        if let Some(ref id) = spine_item.id {
            if id == clean_href {
                return Some(spine_index);
            }
        }
    }

    for (spine_index, spine_item) in epub_doc.spine.iter().enumerate() {
        if let Some((path, _)) = epub_doc.resources.get(&spine_item.idref) {
            let path_str = path.to_string_lossy();

            if path_str == clean_href {
                return Some(spine_index);
            }

            if let Some(filename) = path.file_name() {
                if filename.to_string_lossy() == clean_href {
                    return Some(spine_index);
                }
            }

            if path_str.ends_with(clean_href) {
                return Some(spine_index);
            }
        }
    }

    None
}

// 处理当前章节的图片
fn process_chapter_images(html: &str, epub_doc: &mut EpubDocument) -> String {
    let mut images_map: HashMap<String, String> = HashMap::new();

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

    process_html_images(html, &images_map)
}

// 清理HTML内容
fn clean_html_content(html: &str) -> String {
    let document = Html::parse_document(html);
    let body_selector = Selector::parse("body").unwrap();
    let mut content = String::new();

    let container = document
        .select(&body_selector)
        .next()
        .unwrap_or_else(|| document.root_element());

    extract_text_content(container, &mut content);

    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&content, " ").trim().to_string()
}

// 提取文本内容
fn extract_text_content(element: scraper::ElementRef, content: &mut String) {
    for node in element.children() {
        match node.value() {
            scraper::Node::Text(text) => {
                content.push_str(&text.text);
            }
            scraper::Node::Element(elem) => {
                if elem.name() != "script" && elem.name() != "style" {
                    if let Some(child_element) = scraper::ElementRef::wrap(node) {
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

// 判断是否为块级元素
fn is_block_element(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "br" | "hr" | "blockquote" | "pre"
    )
}

// 简化文本内容提取函数
fn extract_text_content_simple(document: &Html) -> String {
    let body_selector = Selector::parse("body").unwrap();
    let mut text_content = String::new();

    if let Some(body) = document.select(&body_selector).next() {
        extract_text_content(body, &mut text_content);
    } else {
        let root_selector = Selector::parse("*").unwrap();
        for element in document.select(&root_selector) {
            if element.value().name() != "script" && element.value().name() != "style" {
                let text = element.text().collect::<String>();
                if !text.trim().is_empty() {
                    text_content.push_str(&text);
                    text_content.push(' ');
                }
            }
        }
    }

    text_content.trim().to_string()
}

// 从内容中检测标题
fn detect_title_from_content(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }

    for (i, line) in lines.iter().enumerate().take(5) {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.len() < 2 || trimmed.len() > 100 {
            continue;
        }

        if is_valid_chapter_title_enhanced(trimmed) {
            return Some(trimmed.to_string());
        }

        if i == 0 && !contains_common_non_title_patterns(trimmed) {
            if looks_like_title(trimmed) {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

// 检查是否包含常见的非标题模式
fn contains_common_non_title_patterns(text: &str) -> bool {
    let non_title_patterns = vec!["。", "，", "！", "？", ".", ",", "!", "?"];

    for pattern in non_title_patterns {
        if text.contains(pattern) {
            return true;
        }
    }

    false
}

// 判断文本是否看起来像标题
fn looks_like_title(text: &str) -> bool {
    if text.len() > 50 {
        return false;
    }

    let punctuation_count = text.chars().filter(|c| c.is_ascii_punctuation()).count();
    let punctuation_ratio = punctuation_count as f32 / text.len() as f32;

    if punctuation_ratio > 0.3 {
        return false;
    }

    let has_number = text.chars().any(|c| {
        c.is_ascii_digit() || "一二三四五六七八九十百千万零〇壹贰叁肆伍陆柒捌玖拾佰仟萬".contains(c)
    });
    let starts_with_capital = text
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_uppercase());

    has_number || starts_with_capital || text.len() <= 20
}

// 提取文件名中的数字ID
fn extract_number_id(filename: &str) -> String {
    let re = Regex::new(r"(\d+)").unwrap();
    if let Some(caps) = re.captures(filename) {
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

// 提取TOC结构的主函数
fn extract_toc_structure(epub_doc: &mut EpubDocument) -> Result<Vec<TocEntry>, String> {
    if !epub_doc.toc.is_empty() {
        let mut entries = Vec::new();
        for (index, nav_point) in epub_doc.toc.iter().enumerate() {
            entries.push(TocEntry {
                title: nav_point.label.clone(),
                href: nav_point.content.to_string_lossy().to_string(),
                level: 1,
                play_order: Some(index as u32 + 1),
            });
        }
        return Ok(entries);
    }

    Ok(Vec::new())
}

// 共享的EPUB解析基础函数，处理load_epub_file和get_epub_info的共同逻辑
fn extract_epub_chapters_base(
    file_path: &str,
) -> Result<(EpubDocument, Vec<TocEntry>, HashMap<usize, TocEntry>, usize), String> {
    let mut epub_doc = match EpubDoc::new(file_path) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Failed to open EPUB file: {}", e)),
    };

    let toc_entries = extract_toc_structure(&mut epub_doc).unwrap_or_default();
    let spine_to_toc = build_spine_toc_mapping(&mut epub_doc, &toc_entries)?;
    let spine_len = epub_doc.get_num_pages();

    Ok((epub_doc, toc_entries, spine_to_toc, spine_len))
}

// 主要的EPUB加载命令 - 使用增强策略
#[tauri::command]
pub async fn load_epub_file(file_path: String) -> Result<Vec<EpubChapter>, String> {
    let (mut epub_doc, toc_entries, spine_to_toc, spine_len) =
        extract_epub_chapters_base(&file_path)?;

    let mut chapters = Vec::new();

    for i in 0..spine_len {
        epub_doc.set_current_page(i);

        let content = match epub_doc.get_current_str() {
            Some((content, _)) => content,
            None => continue,
        };

        let (title, confidence) = extract_title_with_confidence(&content, &toc_entries);
        let (level, parent_index) = determine_chapter_hierarchy(i, &spine_to_toc, &chapters);

        let detection_method = if confidence > 0.9 {
            "TOC".to_string()
        } else if confidence > 0.7 {
            "HTML_HEADING".to_string()
        } else {
            "HEURISTIC".to_string()
        };

        let processed_html = process_chapter_images(&content, &mut epub_doc);
        let clean_content = clean_html_content(&content);

        chapters.push(EpubChapter {
            title,
            content: clean_content,
            html_content: processed_html,
            level,
            parent_index,
            toc_entry: spine_to_toc.get(&i).map(|entry| entry.title.clone()),
            detection_method,
        });
    }

    if !chapters.is_empty() && chapters[0].title == "未知章节" {
        chapters[0].title = "封面".to_string();
    }

    Ok(chapters)
}

// 获取EPUB基本信息 - 使用增强策略
#[tauri::command]
pub async fn get_epub_info(file_path: String) -> Result<Vec<EpubChapterInfo>, String> {
    let (mut epub_doc, toc_entries, spine_to_toc, spine_len) =
        extract_epub_chapters_base(&file_path)?;

    let mut chapters_info = Vec::new();
    let mut temp_chapters: Vec<EpubChapter> = Vec::new(); // 用于层级计算

    for i in 0..spine_len {
        epub_doc.set_current_page(i);

        let content = match epub_doc.get_current_str() {
            Some((content, _)) => content,
            None => {
                let title = format!("第{}章", i + 1);
                chapters_info.push(EpubChapterInfo {
                    title,
                    index: i,
                    spine_id: i.to_string(),
                    level: 1,
                    parent_index: None,
                    detection_method: "SPINE_FALLBACK".to_string(),
                });
                continue;
            }
        };

        let (title, confidence) = extract_title_with_confidence(&content, &toc_entries);

        let (level, parent_index) = determine_chapter_hierarchy(i, &spine_to_toc, &temp_chapters);

        let detection_method = if confidence > 0.9 {
            "TOC".to_string()
        } else if confidence > 0.7 {
            "HTML_HEADING".to_string()
        } else {
            "HEURISTIC".to_string()
        };

        chapters_info.push(EpubChapterInfo {
            title,
            index: i,
            spine_id: i.to_string(),
            level,
            parent_index,
            detection_method: detection_method.clone(),
        });

        // 更新临时章节并添加到列表
        let updated_temp_chapter = EpubChapter {
            title: chapters_info[i].title.clone(),
            content: String::new(),
            html_content: String::new(),
            level,
            parent_index,
            toc_entry: spine_to_toc.get(&i).map(|entry| entry.title.clone()),
            detection_method,
        };
        temp_chapters.push(updated_temp_chapter);
    }

    if !chapters_info.is_empty() && chapters_info[0].title == "未知章节" {
        chapters_info[0].title = "封面".to_string();
    }

    Ok(chapters_info)
}

// 按需加载单个章节 - 使用增强策略
#[tauri::command]
pub async fn load_epub_chapter(
    file_path: String,
    chapter_index: usize,
) -> Result<EpubChapter, String> {
    let mut epub_doc = match EpubDoc::new(&file_path) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Failed to open EPUB file: {}", e)),
    };

    epub_doc.set_current_page(chapter_index);

    let content = match epub_doc.get_current_str() {
        Some((content, _)) => content,
        None => return Err("Failed to get chapter content".to_string()),
    };

    let toc_entries = extract_toc_structure(&mut epub_doc).unwrap_or_default();
    let (title, confidence) = extract_title_with_confidence(&content, &toc_entries);

    let detection_method = if confidence > 0.9 {
        "TOC".to_string()
    } else if confidence > 0.7 {
        "HTML_HEADING".to_string()
    } else {
        "HEURISTIC".to_string()
    };

    let processed_html = process_chapter_images(&content, &mut epub_doc);
    let clean_content = clean_html_content(&content);

    Ok(EpubChapter {
        title,
        content: clean_content,
        html_content: processed_html,
        level: 1, // 单独加载时默认为1级
        parent_index: None,
        toc_entry: None,
        detection_method,
    })
}
