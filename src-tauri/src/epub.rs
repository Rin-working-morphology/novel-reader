use base64::{engine::general_purpose, Engine as _};
use epub::doc::EpubDoc;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

// 增强的章节结构
#[derive(Debug, Serialize, Deserialize)]
pub struct EpubChapter {
    pub title: String,
    pub content: String,
    pub html_content: String,
    pub href: Option<String>,
    pub anchor: Option<String>,
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
    pub spine_index: usize,
    pub href: Option<String>,
    pub anchor: Option<String>,
    pub level: u8,
    pub parent_index: Option<usize>,
    pub detection_method: String,
}

type EpubDocument = EpubDoc<BufReader<File>>;

// TOC条目结构
#[derive(Debug, Clone)]
struct TocEntry {
    title: String,
    href: Option<String>,
    level: u8,
    parent_toc_index: Option<usize>,
}

#[derive(Debug, Clone)]
struct RawTocEntry {
    title: String,
    href: Option<String>,
    level: u8,
    children: Vec<RawTocEntry>,
}

#[derive(Debug, Clone)]
struct ChapterRef {
    title: String,
    index: usize,
    spine_index: usize,
    spine_id: String,
    href: Option<String>,
    anchor: Option<String>,
    level: u8,
    parent_index: Option<usize>,
    toc_entry: Option<String>,
    detection_method: String,
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

fn build_chapter_refs(epub_doc: &EpubDocument, toc_entries: &[TocEntry]) -> Vec<ChapterRef> {
    if toc_entries.is_empty() {
        return build_spine_fallback_chapter_refs(epub_doc);
    }

    let mut chapter_refs = Vec::new();
    let mut toc_to_chapter_index: HashMap<usize, usize> = HashMap::new();

    for (toc_index, toc_entry) in toc_entries.iter().enumerate() {
        let Some(href) = toc_entry.href.as_deref() else {
            continue;
        };
        let Some(spine_index) = find_spine_index_by_href(epub_doc, href) else {
            continue;
        };

        let parent_index = find_nearest_toc_parent_index(
            toc_entry.parent_toc_index,
            toc_entries,
            &toc_to_chapter_index,
        );
        let index = chapter_refs.len();
        toc_to_chapter_index.insert(toc_index, index);

        let anchor = split_href(href).1.map(|value| value.to_string());

        chapter_refs.push(ChapterRef {
            title: normalize_title(&toc_entry.title)
                .unwrap_or_else(|| format!("第{}章", index + 1)),
            index,
            spine_index,
            spine_id: epub_doc
                .spine
                .get(spine_index)
                .map(|item| item.idref.clone())
                .unwrap_or_else(|| spine_index.to_string()),
            href: Some(href.to_string()),
            anchor,
            level: toc_entry.level.max(1),
            parent_index,
            toc_entry: Some(toc_entry.title.clone()),
            detection_method: "TOC".to_string(),
        });
    }

    if chapter_refs.is_empty() {
        build_spine_fallback_chapter_refs(epub_doc)
    } else {
        chapter_refs
    }
}

fn build_spine_fallback_chapter_refs(epub_doc: &EpubDocument) -> Vec<ChapterRef> {
    epub_doc
        .spine
        .iter()
        .enumerate()
        .filter(|(_, item)| item.linear)
        .enumerate()
        .map(|(index, (spine_index, item))| ChapterRef {
            title: format!("第{}章", index + 1),
            index,
            spine_index,
            spine_id: item.idref.clone(),
            href: epub_doc
                .resources
                .get(&item.idref)
                .map(|(path, _)| path_to_epub_href(path)),
            anchor: None,
            level: 1,
            parent_index: None,
            toc_entry: None,
            detection_method: "SPINE_FALLBACK".to_string(),
        })
        .collect()
}

fn find_nearest_toc_parent_index(
    mut parent_toc_index: Option<usize>,
    toc_entries: &[TocEntry],
    toc_to_chapter_index: &HashMap<usize, usize>,
) -> Option<usize> {
    while let Some(toc_index) = parent_toc_index {
        if let Some(chapter_index) = toc_to_chapter_index.get(&toc_index) {
            return Some(*chapter_index);
        }

        parent_toc_index = toc_entries
            .get(toc_index)
            .and_then(|entry| entry.parent_toc_index);
    }

    None
}

// 改进的spine索引查找函数
fn find_spine_index_by_href(epub_doc: &EpubDocument, href: &str) -> Option<usize> {
    let clean_href = normalize_href_path(href);

    for (spine_index, spine_item) in epub_doc.spine.iter().enumerate() {
        if spine_item.idref == clean_href {
            return Some(spine_index);
        }

        if let Some(ref id) = spine_item.id {
            if id == &clean_href {
                return Some(spine_index);
            }
        }
    }

    for (spine_index, spine_item) in epub_doc.spine.iter().enumerate() {
        if let Some((path, _)) = epub_doc.resources.get(&spine_item.idref) {
            let path_str = normalize_path_string(&path.to_string_lossy());

            if path_str == clean_href {
                return Some(spine_index);
            }

            if let Some(filename) = path.file_name() {
                if filename.to_string_lossy() == clean_href {
                    return Some(spine_index);
                }
            }

            if path_str.ends_with(&clean_href) {
                return Some(spine_index);
            }
        }
    }

    None
}

fn split_href(href: &str) -> (&str, Option<&str>) {
    let without_query = href.split('?').next().unwrap_or(href);
    let mut parts = without_query.splitn(2, '#');
    let path = parts.next().unwrap_or_default();
    let fragment = parts.next().filter(|value| !value.trim().is_empty());
    (path, fragment)
}

fn normalize_href_path(href: &str) -> String {
    let (path, _) = split_href(href);
    normalize_path_string(path)
}

fn normalize_path_string(path: &str) -> String {
    let path = path.replace('\\', "/");
    let mut parts = Vec::new();

    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }

    parts.join("/")
}

fn path_to_epub_href(path: &Path) -> String {
    normalize_path_string(&path.to_string_lossy())
}

fn resolve_relative_href(base_path: &Path, href: &str) -> String {
    let (path_part, fragment) = split_href(href.trim());
    let mut resolved = if path_part.is_empty() {
        PathBuf::from(base_path)
    } else {
        base_path
            .parent()
            .map(|parent| parent.join(path_part))
            .unwrap_or_else(|| PathBuf::from(path_part))
    };

    if resolved.as_os_str().is_empty() {
        resolved = PathBuf::from(path_part);
    }

    let mut normalized = path_to_epub_href(&resolved);
    if let Some(fragment) = fragment {
        normalized.push('#');
        normalized.push_str(fragment);
    }

    normalized
}

fn normalize_title(title: &str) -> Option<String> {
    let title = Regex::new(r"\s+")
        .ok()
        .map(|re| re.replace_all(title, " ").trim().to_string())
        .unwrap_or_else(|| title.trim().to_string());

    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

fn detection_method_from_confidence(confidence: f32) -> String {
    if confidence > 0.9 {
        "TOC".to_string()
    } else if confidence > 0.7 {
        "HTML_HEADING".to_string()
    } else {
        "HEURISTIC".to_string()
    }
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
    let mut raw_entries = extract_epub3_nav_structure(epub_doc);

    if raw_entries.is_empty() {
        raw_entries = extract_ncx_toc_structure(epub_doc);
    }

    if raw_entries.is_empty() {
        return Ok(Vec::new());
    }

    for entry in &mut raw_entries {
        hydrate_missing_toc_href(entry);
    }

    let mut entries = Vec::new();
    flatten_toc_entries(&raw_entries, None, &mut entries);
    Ok(entries)
}

fn extract_ncx_toc_structure(epub_doc: &EpubDocument) -> Vec<RawTocEntry> {
    if !epub_doc.toc.is_empty() {
        let mut entries = Vec::new();
        for nav_point in &epub_doc.toc {
            entries.push(raw_toc_from_nav_point(nav_point, 1));
        }
        return entries;
    }

    Vec::new()
}

fn raw_toc_from_nav_point(nav_point: &epub::doc::NavPoint, level: u8) -> RawTocEntry {
    RawTocEntry {
        title: nav_point.label.clone(),
        href: Some(path_to_epub_href(&nav_point.content)),
        level,
        children: nav_point
            .children
            .iter()
            .map(|child| raw_toc_from_nav_point(child, level.saturating_add(1)))
            .collect(),
    }
}

fn extract_epub3_nav_structure(epub_doc: &mut EpubDocument) -> Vec<RawTocEntry> {
    let resources = epub_doc.resources.clone();
    let mut candidates: Vec<(String, PathBuf, String)> = resources
        .iter()
        .filter(|(id, (path, mime))| {
            let id = id.to_lowercase();
            let filename = path
                .file_name()
                .map(|value| value.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            is_html_mime(mime)
                && (id.contains("nav")
                    || filename.contains("nav")
                    || filename.contains("toc")
                    || filename.contains("contents"))
        })
        .map(|(id, (path, mime))| (id.clone(), path.clone(), mime.clone()))
        .collect();

    candidates.sort_by_key(|(id, path, _)| {
        let filename = path
            .file_name()
            .map(|value| value.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if id.eq_ignore_ascii_case("nav") || filename.starts_with("nav.") {
            0
        } else if id.contains("toc") || filename.contains("toc") {
            1
        } else {
            2
        }
    });

    for (id, path, _) in candidates {
        if let Some((content, _)) = epub_doc.get_resource_str(&id) {
            let entries = parse_nav_document(&content, &path);
            if !entries.is_empty() {
                return entries;
            }
        }
    }

    Vec::new()
}

fn is_html_mime(mime: &str) -> bool {
    matches!(
        mime,
        "application/xhtml+xml" | "text/html" | "application/x-dtbncx+xml"
    ) || mime.ends_with("+html")
}

fn parse_nav_document(html: &str, base_path: &Path) -> Vec<RawTocEntry> {
    let document = Html::parse_document(html);
    let nav_selector = Selector::parse("nav").unwrap();
    let list_selector = Selector::parse("ol, ul").unwrap();

    let mut fallback_nav = None;
    for nav in document.select(&nav_selector) {
        if is_toc_nav(nav.value()) {
            if let Some(list) = nav.select(&list_selector).next() {
                return parse_nav_list(list, base_path, 1);
            }
        }

        if fallback_nav.is_none() {
            fallback_nav = Some(nav);
        }
    }

    if let Some(nav) = fallback_nav {
        if let Some(list) = nav.select(&list_selector).next() {
            return parse_nav_list(list, base_path, 1);
        }
    }

    Vec::new()
}

fn is_toc_nav(element: &scraper::node::Element) -> bool {
    let epub_type = element.attr("epub:type").unwrap_or_default();
    let role = element.attr("role").unwrap_or_default();
    let id = element.attr("id").unwrap_or_default();
    let class_name = element.attr("class").unwrap_or_default();
    let combined = format!("{} {} {} {}", epub_type, role, id, class_name).to_lowercase();

    combined.contains("toc") || combined.contains("contents") || combined.contains("doc-toc")
}

fn parse_nav_list(list: scraper::ElementRef, base_path: &Path, level: u8) -> Vec<RawTocEntry> {
    let mut entries = Vec::new();

    for child in list.children() {
        let Some(element) = scraper::ElementRef::wrap(child) else {
            continue;
        };

        if element.value().name() == "li" {
            if let Some(entry) = parse_nav_item(element, base_path, level) {
                entries.push(entry);
            }
        }
    }

    entries
}

fn parse_nav_item(item: scraper::ElementRef, base_path: &Path, level: u8) -> Option<RawTocEntry> {
    let mut title = None;
    let mut href = None;
    let mut children = Vec::new();

    for child in item.children() {
        let Some(element) = scraper::ElementRef::wrap(child) else {
            continue;
        };

        match element.value().name() {
            "a" => {
                if title.is_none() {
                    title = normalize_title(&element.text().collect::<String>());
                }

                if href.is_none() {
                    href = element
                        .value()
                        .attr("href")
                        .map(|value| resolve_relative_href(base_path, value));
                }
            }
            "span" => {
                if title.is_none() {
                    title = normalize_title(&element.text().collect::<String>());
                }
            }
            "ol" | "ul" => {
                children.extend(parse_nav_list(element, base_path, level.saturating_add(1)));
            }
            _ => {}
        }
    }

    let title = title.or_else(|| extract_nav_item_text(item))?;

    Some(RawTocEntry {
        title,
        href,
        level,
        children,
    })
}

fn extract_nav_item_text(item: scraper::ElementRef) -> Option<String> {
    for child in item.children() {
        let Some(element) = scraper::ElementRef::wrap(child) else {
            continue;
        };

        if matches!(element.value().name(), "ol" | "ul") {
            continue;
        }

        if let Some(title) = normalize_title(&element.text().collect::<String>()) {
            return Some(title);
        }
    }

    None
}

fn hydrate_missing_toc_href(entry: &mut RawTocEntry) -> Option<String> {
    for child in &mut entry.children {
        hydrate_missing_toc_href(child);
    }

    if entry.href.is_none() {
        entry.href = entry
            .children
            .iter()
            .find_map(|child| child.href.as_ref().cloned());
    }

    entry.href.clone()
}

fn flatten_toc_entries(
    raw_entries: &[RawTocEntry],
    parent_index: Option<usize>,
    entries: &mut Vec<TocEntry>,
) {
    for raw_entry in raw_entries {
        let current_index = entries.len();
        entries.push(TocEntry {
            title: raw_entry.title.clone(),
            href: raw_entry.href.clone(),
            level: raw_entry.level,
            parent_toc_index: parent_index,
        });

        flatten_toc_entries(&raw_entry.children, Some(current_index), entries);
    }
}

// 共享的EPUB解析基础函数，处理load_epub_file和get_epub_info的共同逻辑
fn extract_epub_chapters_base(
    file_path: &str,
) -> Result<(EpubDocument, Vec<TocEntry>, Vec<ChapterRef>), String> {
    let mut epub_doc = match EpubDoc::new(file_path) {
        Ok(doc) => doc,
        Err(e) => return Err(format!("Failed to open EPUB file: {}", e)),
    };

    let toc_entries = extract_toc_structure(&mut epub_doc).unwrap_or_default();
    let chapter_refs = build_chapter_refs(&epub_doc, &toc_entries);

    Ok((epub_doc, toc_entries, chapter_refs))
}

// 主要的EPUB加载命令 - 使用增强策略
#[tauri::command]
pub async fn load_epub_file(file_path: String) -> Result<Vec<EpubChapter>, String> {
    let (mut epub_doc, toc_entries, chapter_refs) = extract_epub_chapters_base(&file_path)?;

    let mut chapters = Vec::new();

    for chapter_ref in chapter_refs {
        epub_doc.set_current_page(chapter_ref.spine_index);

        let content = match epub_doc.get_current_str() {
            Some((content, _)) => content,
            None => continue,
        };

        let (title, confidence) = extract_title_with_confidence(&content, &toc_entries);
        let title = if chapter_ref.detection_method == "TOC" {
            chapter_ref.title.clone()
        } else {
            title
        };
        let detection_method = if chapter_ref.detection_method == "TOC" {
            "TOC".to_string()
        } else {
            detection_method_from_confidence(confidence)
        };

        let processed_html = process_chapter_images(&content, &mut epub_doc);
        let clean_content = clean_html_content(&content);

        chapters.push(EpubChapter {
            title,
            content: clean_content,
            html_content: processed_html,
            href: chapter_ref.href,
            anchor: chapter_ref.anchor,
            level: chapter_ref.level,
            parent_index: chapter_ref.parent_index,
            toc_entry: chapter_ref.toc_entry,
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
    let (mut epub_doc, toc_entries, chapter_refs) = extract_epub_chapters_base(&file_path)?;

    let mut chapters_info = Vec::new();

    for chapter_ref in chapter_refs {
        epub_doc.set_current_page(chapter_ref.spine_index);

        let content = match epub_doc.get_current_str() {
            Some((content, _)) => content,
            None => {
                chapters_info.push(EpubChapterInfo {
                    title: chapter_ref.title,
                    index: chapter_ref.index,
                    spine_id: chapter_ref.spine_id,
                    spine_index: chapter_ref.spine_index,
                    href: chapter_ref.href,
                    anchor: chapter_ref.anchor,
                    level: chapter_ref.level,
                    parent_index: chapter_ref.parent_index,
                    detection_method: chapter_ref.detection_method,
                });
                continue;
            }
        };

        let (title, confidence) = extract_title_with_confidence(&content, &toc_entries);
        let (title, detection_method) = if chapter_ref.detection_method == "TOC" {
            (chapter_ref.title.clone(), "TOC".to_string())
        } else {
            (title, detection_method_from_confidence(confidence))
        };

        chapters_info.push(EpubChapterInfo {
            title,
            index: chapter_ref.index,
            spine_id: chapter_ref.spine_id,
            spine_index: chapter_ref.spine_index,
            href: chapter_ref.href,
            anchor: chapter_ref.anchor,
            level: chapter_ref.level,
            parent_index: chapter_ref.parent_index,
            detection_method,
        });
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
    let (mut epub_doc, toc_entries, chapter_refs) = extract_epub_chapters_base(&file_path)?;
    let chapter_ref = chapter_refs
        .get(chapter_index)
        .cloned()
        .ok_or_else(|| "Chapter index out of range".to_string())?;

    epub_doc.set_current_page(chapter_ref.spine_index);

    let content = match epub_doc.get_current_str() {
        Some((content, _)) => content,
        None => return Err("Failed to get chapter content".to_string()),
    };

    let (title, confidence) = extract_title_with_confidence(&content, &toc_entries);
    let (title, detection_method) = if chapter_ref.detection_method == "TOC" {
        (chapter_ref.title.clone(), "TOC".to_string())
    } else if confidence > 0.7 {
        (title, "HTML_HEADING".to_string())
    } else {
        (title, "HEURISTIC".to_string())
    };

    let processed_html = process_chapter_images(&content, &mut epub_doc);
    let clean_content = clean_html_content(&content);

    Ok(EpubChapter {
        title,
        content: clean_content,
        html_content: processed_html,
        href: chapter_ref.href,
        anchor: chapter_ref.anchor,
        level: chapter_ref.level,
        parent_index: chapter_ref.parent_index,
        toc_entry: chapter_ref.toc_entry,
        detection_method,
    })
}
