// src/explorer/state.rs

use super::layout::ExplorerLayout;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: u64,
    pub is_hidden: bool,
    pub is_executable: bool,
    pub is_readonly: bool,
}

#[derive(Clone, PartialEq)]
pub enum SortMode {
    Name,
    Size,
    Modified,
    Type,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Name => "name",
            SortMode::Size => "size",
            SortMode::Modified => "modified",
            SortMode::Type => "type",
        }
    }
}

#[derive(Clone)]
pub enum Dialog {
    NewFile,
    NewDir,
    Rename,
}

#[derive(Clone)]
pub enum Confirm {
    Delete { name: String, is_dir: bool },
    #[allow(dead_code)]
    Overwrite { src: PathBuf, dest: PathBuf },
}

#[derive(Clone)]
pub struct Opener {
    pub name: String,
    pub command: String,
    pub icon: String,
}

#[derive(Clone, PartialEq)]
pub enum PreviewKind {
    Text,
    Image,
    Video,
    Audio,
    Binary,
    TooLarge,
    Directory,
    Empty,
}

#[derive(Clone)]
pub struct MountedDrive {
    pub name: String,
    pub path: PathBuf,
    pub size_total: u64,
    #[allow(dead_code)]
    pub size_used: u64,
    pub icon: String,
}

pub struct ExplorerState {
    pub cwd: PathBuf,
    pub entries: Vec<FileEntry>,
    pub all_entries: Vec<FileEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub show_hidden: bool,
    pub sort_mode: SortMode,
    pub filter: String,
    pub filter_active: bool,
    pub dialog: Option<Dialog>,
    pub dialog_input: String,
    pub confirm: Option<Confirm>,
    pub status_msg: Option<String>,
    pub yanked: Option<PathBuf>,
    pub cut_mode: bool,
    pub show_preview: bool,
    pub history: Vec<PathBuf>,
    pub tick: u64,
    pub show_open_with: bool,
    pub open_with_selected: usize,
    pub openers: Vec<Opener>,
    preview_cache_path: Option<PathBuf>,
    preview_cache_lines: Vec<String>,
    preview_cache_kind: PreviewKind,
    preview_cache_info: Vec<String>,
    pub mounted_drives: Vec<MountedDrive>,
    pub layout: ExplorerLayout,
}

impl ExplorerState {
    pub fn new(start: &PathBuf) -> Self {
        let openers = detect_openers();
        let mounted_drives = detect_mounted_drives();
        Self {
            cwd: start.clone(),
            entries: vec![],
            all_entries: vec![],
            selected: 0,
            scroll_offset: 0,
            show_hidden: false,
            sort_mode: SortMode::Name,
            filter: String::new(),
            filter_active: false,
            dialog: None,
            dialog_input: String::new(),
            confirm: None,
            status_msg: None,
            yanked: None,
            cut_mode: false,
            show_preview: false,
            history: vec![],
            tick: 0,
            show_open_with: false,
            open_with_selected: 0,
            openers,
            preview_cache_path: None,
            preview_cache_lines: vec![],
            preview_cache_kind: PreviewKind::Empty,
            preview_cache_info: vec![],
            mounted_drives,
            layout: ExplorerLayout::default(),
        }
    }

    pub fn refresh(&mut self) {
        self.all_entries = read_dir(&self.cwd, self.show_hidden);
        let mode = self.sort_mode.clone();
        sort_entries(&mut self.all_entries, &mode);
        self.apply_filter();
        self.clamp_selected();
        self.invalidate_preview();
        self.mounted_drives = detect_mounted_drives();
    }

    pub fn apply_filter(&mut self) {
        if self.filter.is_empty() {
            self.entries = self.all_entries.clone();
        } else {
            let q = self.filter.to_lowercase();
            self.entries = self.all_entries
                .iter()
                .filter(|e| fuzzy_match(&e.name.to_lowercase(), &q))
                .cloned()
                .collect();
        }
        if let Some(parent) = self.cwd.parent() {
            self.entries.insert(
                0,
                FileEntry {
                    name: "..".to_string(),
                    path: parent.to_path_buf(),
                    is_dir: true,
                    is_symlink: false,
                    size: 0,
                    modified: 0,
                    is_hidden: false,
                    is_executable: false,
                    is_readonly: false,
                },
            );
        }
    }

    pub fn is_parent_entry(entry: &FileEntry) -> bool {
        entry.name == ".." && entry.is_dir
    }

    pub fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Name => SortMode::Size,
            SortMode::Size => SortMode::Modified,
            SortMode::Modified => SortMode::Type,
            SortMode::Type => SortMode::Name,
        };
        self.status_msg = Some(format!("sort: {}", self.sort_mode.label()));
    }

    fn clamp_selected(&mut self) {
        if self.entries.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.entries.is_empty() { return; }
        if self.selected == 0 { self.selected = self.entries.len() - 1; }
        else { self.selected -= 1; }
        self.invalidate_preview();
    }

    pub fn move_down(&mut self) {
        if self.entries.is_empty() { return; }
        self.selected = (self.selected + 1) % self.entries.len();
        self.invalidate_preview();
    }

    pub fn page_up(&mut self, n: usize) {
        self.selected = self.selected.saturating_sub(n);
        self.invalidate_preview();
    }

    pub fn page_down(&mut self, n: usize) {
        if self.entries.is_empty() { return; }
        self.selected = (self.selected + n).min(self.entries.len() - 1);
        self.invalidate_preview();
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 { return; }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected - visible_height + 1;
        }
    }

    pub fn selected_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.selected)
    }

    pub fn enter_dir(&mut self, path: &PathBuf) {
        self.history.push(self.cwd.clone());
        self.cwd = path.clone();
        self.selected = 0;
        self.scroll_offset = 0;
        self.filter.clear();
        self.filter_active = false;
        self.refresh();
        if self.cwd.parent().is_some() && self.entries.len() > 1 {
            self.selected = 1;
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            let old_name = self.cwd.file_name().map(|n| n.to_string_lossy().to_string());
            self.history.push(self.cwd.clone());
            self.cwd = parent.to_path_buf();
            self.filter.clear();
            self.filter_active = false;
            self.refresh();
            if let Some(name) = old_name {
                if let Some(idx) = self
                    .entries
                    .iter()
                    .position(|e| e.name == name && !Self::is_parent_entry(e))
                {
                    self.selected = idx;
                }
            }
        }
    }

    pub fn show_open_with_menu(&mut self) {
        if self.selected_entry().is_some() {
            self.show_open_with = true;
            self.open_with_selected = 0;
        }
    }

    pub fn open_with(&mut self, opener_idx: usize) -> Option<String> {
        let entry = self.selected_entry()?.clone();
        let opener = self.openers.get(opener_idx)?.clone();
        let path_str = entry.path.to_string_lossy().to_string();
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("{} \"{}\"", opener.command, path_str))
            .spawn();
        match result {
            Ok(_) => Some(format!("opened with {}", opener.name)),
            Err(e) => Some(format!("failed: {}", e)),
        }
    }

    pub fn invalidate_preview(&mut self) {
        let current = self.selected_entry().map(|e| e.path.clone());
        if current != self.preview_cache_path {
            self.preview_cache_path = current;
            self.preview_cache_lines.clear();
            self.preview_cache_kind = PreviewKind::Empty;
            self.preview_cache_info.clear();
        }
    }

    pub fn start_filter(&mut self) {
        self.filter_active = true;
        self.filter.clear();
    }

    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.apply_filter();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.apply_filter();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn end_filter(&mut self) {
        self.filter_active = false;
        if self.filter.is_empty() {
            self.entries = self.all_entries.clone();
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.filter_active = false;
        self.entries = self.all_entries.clone();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn get_preview(&mut self, width: usize, height: usize) -> (&[String], &PreviewKind, &[String]) {
        let entry = match self.selected_entry() {
            Some(e) => e.clone(),
            None => {
                self.preview_cache_kind = PreviewKind::Empty;
                self.preview_cache_lines = vec!["(no selection)".to_string()];
                return (&self.preview_cache_lines, &self.preview_cache_kind, &self.preview_cache_info);
            }
        };

        if !self.preview_cache_lines.is_empty() {
            return (&self.preview_cache_lines, &self.preview_cache_kind, &self.preview_cache_info);
        }

        if entry.is_dir {
            self.preview_cache_kind = PreviewKind::Directory;
            self.preview_cache_lines = preview_directory(&entry);
            self.preview_cache_info = vec![
                format!("📁 Directory: {}", entry.name),
                format!("📍 {}", entry.path.display()),
            ];
        } else {
            let ext = entry.name.rsplit('.').next().unwrap_or("").to_lowercase();
            let kind = classify_file(&ext);
            self.preview_cache_kind = kind.clone();

            match kind {
                PreviewKind::Image => {
                    self.preview_cache_lines = preview_image(&entry.path, width, height);
                    self.preview_cache_info = image_info(&entry);
                }
                PreviewKind::Video => {
                    self.preview_cache_lines = preview_video(&entry.path, width, height);
                    self.preview_cache_info = media_info(&entry, "🎬 Video");
                }
                PreviewKind::Audio => {
                    self.preview_cache_lines = preview_audio(&entry);
                    self.preview_cache_info = media_info(&entry, "🎵 Audio");
                }
                PreviewKind::TooLarge => {
                    self.preview_cache_lines = vec![
                        format!("(file too large: {})", crate::utils::format_size(entry.size)),
                    ];
                    self.preview_cache_info = vec![format!("💾 {}", crate::utils::format_size(entry.size))];
                }
                PreviewKind::Binary => {
                    self.preview_cache_lines = preview_binary(&entry.path);
                    self.preview_cache_info = vec![
                        "⚙️ Binary file".to_string(),
                        format!("💾 {}", crate::utils::format_size(entry.size)),
                    ];
                }
                PreviewKind::Text => {
                    if entry.size > 2 * 1024 * 1024 {
                        self.preview_cache_lines = vec!["(too large for preview)".to_string()];
                        self.preview_cache_kind = PreviewKind::TooLarge;
                    } else {
                        self.preview_cache_lines = preview_text(&entry.path, height);
                    }
                    self.preview_cache_info = vec![
                        format!("📄 {}", entry.name),
                        format!("💾 {}", crate::utils::format_size(entry.size)),
                    ];
                }
                _ => {
                    self.preview_cache_lines = vec!["(no preview)".to_string()];
                    self.preview_cache_info = vec![];
                }
            }
        }

        (&self.preview_cache_lines, &self.preview_cache_kind, &self.preview_cache_info)
    }
}

// ── Free functions ─────────────────────────────────────────────────────────────

fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() { return true; }
    if haystack.contains(needle) { return true; }
    let initials: String = haystack
        .split(|c: char| c == '-' || c == '_' || c == '.' || c == ' ')
        .filter_map(|seg| seg.chars().next())
        .collect();
    if initials.contains(needle) { return true; }
    let mut chars = needle.chars().peekable();
    for h in haystack.chars() {
        if chars.peek().map(|c| *c == h).unwrap_or(false) {
            chars.next();
        }
    }
    chars.peek().is_none()
}

fn sort_entries(entries: &mut Vec<FileEntry>, sort_mode: &SortMode) {
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }
        match sort_mode {
            SortMode::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortMode::Size => b.size.cmp(&a.size),
            SortMode::Modified => b.modified.cmp(&a.modified),
            SortMode::Type => {
                let ext_a = a.name.rsplit('.').next().unwrap_or("");
                let ext_b = b.name.rsplit('.').next().unwrap_or("");
                ext_a.cmp(ext_b).then(a.name.cmp(&b.name))
            }
        }
    });
}

fn classify_file(ext: &str) -> PreviewKind {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "webp"
        | "tiff" | "tif" | "ppm" | "pgm" | "pbm" | "pnm" | "svg" => PreviewKind::Image,
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "webm" | "flv"
        | "m4v" | "ogv" | "3gp" | "ts" | "mts" => PreviewKind::Video,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "oga" | "opus"
        | "wma" | "m4a" | "mid" | "midi" | "alac" => PreviewKind::Audio,
        "bin" | "exe" | "dll" | "so" | "dylib" | "o" | "a" | "lib"
        | "wasm" | "class" | "pyc" | "pyo" | "elc"
        | "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst"
        | "deb" | "rpm" | "dmg" | "iso" | "pkg" | "apk" | "msi"
        | "ttf" | "otf" | "woff" | "woff2" | "eot"
        | "sqlite" | "db" => PreviewKind::Binary,
        _ => PreviewKind::Text,
    }
}

fn preview_image(path: &PathBuf, term_width: usize, term_height: usize) -> Vec<String> {
    if let Some(lines) = preview_image_chafa(path, term_width, term_height) {
        return lines;
    }
    if let Some(lines) = preview_image_viu(path, term_width, term_height) {
        return lines;
    }
    preview_image_builtin(path, term_width, term_height)
}

fn preview_image_chafa(path: &PathBuf, width: usize, height: usize) -> Option<Vec<String>> {
    let output = std::process::Command::new("chafa")
        .args([
            "--size", &format!("{}x{}", width.saturating_sub(2), height.saturating_sub(2)),
            "--colors", "full",
            "--symbols", "block+border+space",
            "--fill", "block+border+space",
            &path.to_string_lossy(),
        ])
        .output().ok()?;
    if !output.status.success() { return None; }
    Some(String::from_utf8_lossy(&output.stdout).lines().map(|l| l.to_string()).collect())
}

fn preview_image_viu(path: &PathBuf, width: usize, height: usize) -> Option<Vec<String>> {
    let output = std::process::Command::new("viu")
        .args(["-w", &width.saturating_sub(2).to_string(), "-h", &height.saturating_sub(2).to_string(), &path.to_string_lossy()])
        .output().ok()?;
    if !output.status.success() { return None; }
    Some(String::from_utf8_lossy(&output.stdout).lines().map(|l| l.to_string()).collect())
}

fn preview_image_builtin(path: &PathBuf, term_width: usize, term_height: usize) -> Vec<String> {
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => return vec![format!("❌ cannot open: {}", e)],
    };
    let (orig_w, orig_h) = (img.width(), img.height());
    let max_w = term_width.saturating_sub(4).max(10);
    let max_h = (term_height.saturating_sub(4)).max(4) * 2;
    let scale_w = max_w as f64 / orig_w as f64;
    let scale_h = max_h as f64 / orig_h as f64;
    let scale = scale_w.min(scale_h).min(1.0);
    let new_w = (orig_w as f64 * scale).max(1.0) as u32;
    let mut new_h = (orig_h as f64 * scale).max(2.0) as u32;
    if new_h % 2 != 0 { new_h += 1; }

    let resized = image::imageops::resize(
        &img.to_rgba8(), new_w, new_h, image::imageops::FilterType::Lanczos3,
    );

    let mut lines = Vec::new();
    let mut y = 0u32;
    while y < new_h {
        let mut line = String::new();
        for x in 0..new_w {
            let top = resized.get_pixel(x, y);
            let bottom = if y + 1 < new_h { resized.get_pixel(x, y + 1) } else { top };
            if top[3] < 30 && bottom[3] < 30 {
                line.push(' ');
            } else if top[3] < 30 {
                line.push_str(&format!("\x1b[38;2;{};{};{}m▄\x1b[0m", bottom[0], bottom[1], bottom[2]));
            } else if bottom[3] < 30 {
                line.push_str(&format!("\x1b[38;2;{};{};{}m▀\x1b[0m", top[0], top[1], top[2]));
            } else {
                line.push_str(&format!(
                    "\x1b[38;2;{};{};{};48;2;{};{};{}m▀\x1b[0m",
                    top[0], top[1], top[2], bottom[0], bottom[1], bottom[2]
                ));
            }
        }
        lines.push(line);
        y += 2;
    }
    lines
}

fn image_info(entry: &FileEntry) -> Vec<String> {
    let mut info = vec![
        format!("🖼️ {}", entry.name),
        format!("💾 {}", crate::utils::format_size(entry.size)),
    ];
    if let Ok(img) = image::open(&entry.path) {
        info.push(format!("📐 {}×{} px", img.width(), img.height()));
        let mp = (img.width() as f64 * img.height() as f64) / 1_000_000.0;
        if mp >= 0.1 { info.push(format!("📷 {:.1} MP", mp)); }
    }
    let ext = entry.name.rsplit('.').next().unwrap_or("").to_uppercase();
    info.push(format!("📎 {}", ext));
    info
}

fn preview_video(path: &PathBuf, width: usize, height: usize) -> Vec<String> {
    let thumb_path = std::env::temp_dir().join("jump_video_thumb.png");
    let result = std::process::Command::new("ffmpeg")
        .args(["-y", "-i", &path.to_string_lossy(), "-ss", "00:00:02", "-vframes", "1",
            "-vf", &format!("scale={}:-1", width * 3), &thumb_path.to_string_lossy()])
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();

    match result {
        Ok(s) if s.success() && thumb_path.exists() => {
            let mut lines = vec!["▶ Video Preview:".to_string(), String::new()];
            lines.extend(preview_image(&thumb_path, width, height.saturating_sub(4)));
            let _ = fs::remove_file(&thumb_path);
            lines
        }
        _ => {
            let mut lines = vec![
                "🎬 Video File".to_string(), String::new(),
                format!("📄 {}", path.file_name().unwrap_or_default().to_string_lossy()),
                format!("💾 {}", crate::utils::format_size(fs::metadata(path).map(|m| m.len()).unwrap_or(0))),
                String::new(),
            ];
            if let Some(info) = get_ffprobe_info(path) { lines.extend(info); }
            else { lines.push("ℹ️ Install ffmpeg for thumbnails".to_string()); }
            lines.push(String::new());
            lines.push("    ╭───────────────╮".to_string());
            lines.push("    │   ▶  PLAY     │".to_string());
            lines.push("    ╰───────────────╯".to_string());
            lines.push(String::new());
            lines.push("  Enter or 'o' to open".to_string());
            lines
        }
    }
}

fn get_ffprobe_info(path: &PathBuf) -> Option<Vec<String>> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", &path.to_string_lossy()])
        .output().ok()?;
    if !output.status.success() { return None; }
    let json: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).ok()?;
    let mut info = Vec::new();

    if let Some(fmt) = json.get("format") {
        if let Some(dur) = fmt.get("duration").and_then(|d| d.as_str()).and_then(|d| d.parse::<f64>().ok()) {
            let h = (dur / 3600.0) as u32;
            let m = ((dur % 3600.0) / 60.0) as u32;
            let s = (dur % 60.0) as u32;
            info.push(if h > 0 { format!("⏱️ {}:{:02}:{:02}", h, m, s) } else { format!("⏱️ {}:{:02}", m, s) });
        }
        if let Some(br) = fmt.get("bit_rate").and_then(|b| b.as_str()).and_then(|b| b.parse::<u64>().ok()) {
            info.push(format!("📊 {} kbps", br / 1000));
        }
    }
    if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
        for s in streams {
            let ct = s.get("codec_type").and_then(|t| t.as_str()).unwrap_or("");
            let cn = s.get("codec_name").and_then(|c| c.as_str()).unwrap_or("?");
            match ct {
                "video" => {
                    let w = s.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                    let h = s.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                    info.push(format!("🎥 {} {}×{}", cn.to_uppercase(), w, h));
                    if let Some(fps) = s.get("r_frame_rate").and_then(|f| f.as_str()) {
                        let p: Vec<&str> = fps.split('/').collect();
                        if p.len() == 2 {
                            if let (Ok(n), Ok(d)) = (p[0].parse::<f64>(), p[1].parse::<f64>()) {
                                if d > 0.0 { info.push(format!("🎞️ {:.1} fps", n / d)); }
                            }
                        }
                    }
                }
                "audio" => {
                    let ch = s.get("channels").and_then(|c| c.as_u64()).unwrap_or(0);
                    let sr = s.get("sample_rate").and_then(|s| s.as_str()).unwrap_or("?");
                    let label = match ch { 1 => "mono", 2 => "stereo", 6 => "5.1", 8 => "7.1", _ => "multi" };
                    info.push(format!("🔊 {} {} {}Hz", cn.to_uppercase(), label, sr));
                }
                "subtitle" => {
                    let lang = s.get("tags").and_then(|t| t.get("language")).and_then(|l| l.as_str()).unwrap_or("?");
                    info.push(format!("💬 Sub: {}", lang));
                }
                _ => {}
            }
        }
    }
    Some(info)
}

fn media_info(entry: &FileEntry, label: &str) -> Vec<String> {
    let mut info = vec![
        format!("{}: {}", label, entry.name),
        format!("💾 {}", crate::utils::format_size(entry.size)),
    ];
    let ext = entry.name.rsplit('.').next().unwrap_or("").to_uppercase();
    info.push(format!("📎 {}", ext));
    if let Some(probe) = get_ffprobe_info(&entry.path) {
        info.push(String::new());
        info.extend(probe);
    }
    info
}

fn preview_audio(entry: &FileEntry) -> Vec<String> {
    let mut lines = vec![
        "🎵 Audio File".to_string(), String::new(),
        format!("📄 {}", entry.name),
        format!("💾 {}", crate::utils::format_size(entry.size)),
        String::new(),
    ];
    if let Some(info) = get_ffprobe_info(&entry.path) { lines.extend(info); lines.push(String::new()); }

    let output = std::process::Command::new("ffprobe")
        .args(["-v", "quiet", "-print_format", "json", "-show_format", &entry.path.to_string_lossy()])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&output.stdout)) {
                if let Some(tags) = json.get("format").and_then(|f| f.get("tags")) {
                    let tag_map = [("title", "🎵 Title"), ("artist", "🎤 Artist"), ("album", "💿 Album"),
                        ("date", "📅 Year"), ("genre", "🏷️ Genre"), ("track", "🔢 Track")];
                    for (key, label) in tag_map {
                        if let Some(val) = tags.get(key).and_then(|t| t.as_str()) {
                            lines.push(format!("{}: {}", label, val));
                        }
                    }
                }
            }
        }
    }

    lines.push(String::new());
    lines.push("  ╭──────────────────────────╮".to_string());
    lines.push("  │ ▁▂▃▅▇█▇▅▃▂▁▂▃▅▇▅▃▂▁▂▃▅ │".to_string());
    lines.push("  │ ▇▅▃▂▁▂▃▅▇█▇▅▃▂▁▂▃▅▇▅▃▁ │".to_string());
    lines.push("  ╰──────────────────────────╯".to_string());
    lines.push(String::new());
    lines.push("  Enter or 'o' to play".to_string());
    lines
}

fn preview_text(path: &PathBuf, max_lines: usize) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => content.lines().take(max_lines + 10).map(|l| l.to_string()).collect(),
        Err(_) => vec!["(cannot read)".to_string()],
    }
}

fn preview_binary(path: &PathBuf) -> Vec<String> {
    let mut lines = vec!["⚙️ Binary — hex dump:".to_string(), String::new()];
    match fs::read(path) {
        Ok(bytes) => {
            let show = bytes.len().min(256);
            for start in (0..show).step_by(16) {
                let end = (start + 16).min(show);
                let chunk = &bytes[start..end];
                let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                let ascii: String = chunk.iter().map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { '.' }).collect();
                lines.push(format!("  {:06x}  {:<48}  {}", start, hex, ascii));
            }
            if bytes.len() > 256 { lines.push(format!("  ... {} more bytes", bytes.len() - 256)); }
        }
        Err(e) => lines.push(format!("  error: {}", e)),
    }
    lines
}

fn preview_directory(entry: &FileEntry) -> Vec<String> {
    match fs::read_dir(&entry.path) {
        Ok(rd) => {
            let mut lines = vec![format!("📁 {}", entry.name), String::new()];
            let mut items: Vec<(bool, String, u64)> = Vec::new();
            let mut dirs = 0u32;
            let mut files = 0u32;
            let mut total: u64 = 0;
            for e in rd.flatten() {
                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                if is_dir { dirs += 1; } else { files += 1; }
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                total += size;
                items.push((is_dir, e.file_name().to_string_lossy().to_string(), size));
            }
            items.sort_by(|a, b| match (a.0, b.0) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
            });
            for (is_dir, name, size) in items.iter().take(25) {
                let icon = if *is_dir { "📁" } else { "📄" };
                let ss = if *is_dir { String::new() } else { format!("  {}", crate::utils::format_size(*size)) };
                lines.push(format!("  {} {}{}", icon, name, ss));
            }
            if items.len() > 25 { lines.push(format!("  ... {} more", items.len() - 25)); }
            lines.push(String::new());
            lines.push(format!("📁 {}  📄 {}  💾 {}", dirs, files, crate::utils::format_size(total)));
            lines
        }
        Err(e) => vec![format!("error: {}", e)],
    }
}

pub fn detect_mounted_drives() -> Vec<MountedDrive> {
    let mut drives = Vec::new();
    let user_media = std::env::var("USER")
        .map(|u| format!("/run/media/{}", u))
        .unwrap_or_else(|_| "/run/media".to_string());

    let mount_points: Vec<(String, String)> = vec![
        ("/media".to_string(), "🔌".to_string()),
        ("/mnt".to_string(), "💾".to_string()),
        (user_media, "🔌".to_string()),
    ];

    for (base, icon) in &mount_points {
        if let Ok(rd) = fs::read_dir(base) {
            for entry in rd.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path = entry.path();
                    let (total, used) = get_disk_usage(&path);
                    drives.push(MountedDrive {
                        name, path, size_total: total, size_used: used, icon: icon.clone(),
                    });
                }
            }
        }
    }
    drives
}

fn get_disk_usage(path: &PathBuf) -> (u64, u64) {
    let output = std::process::Command::new("df")
        .args(["--output=size,used", "-B1", &path.to_string_lossy()])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let lines: Vec<&str> = text.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 2 {
                    let total = parts[0].parse::<u64>().unwrap_or(0);
                    let used = parts[1].parse::<u64>().unwrap_or(0);
                    return (total, used);
                }
            }
            (0, 0)
        }
        _ => (0, 0),
    }
}

fn detect_openers() -> Vec<Opener> {
    let candidates = vec![
        ("VS Code", "code", "💻"), ("VS Code Insiders", "code-insiders", "💻"),
        ("Cursor", "cursor", "💻"), ("Zed", "zed", "💻"),
        ("Neovim", "nvim", "📝"), ("Vim", "vim", "📝"),
        ("Nano", "nano", "📝"), ("Helix", "hx", "📝"),
        ("Emacs", "emacs", "📝"), ("Micro", "micro", "📝"),
        ("Sublime Text", "subl", "💻"),
        ("IntelliJ IDEA", "idea", "☕"), ("PyCharm", "pycharm", "🐍"),
        ("WebStorm", "webstorm", "🌐"), ("CLion", "clion", "⚙️"),
        ("GoLand", "goland", "🔷"), ("RustRover", "rustrover", "🦀"),
        ("Kate", "kate", "📝"), ("Gedit", "gedit", "📝"), ("Mousepad", "mousepad", "📝"),
        ("Thunar", "thunar", "📂"), ("Nautilus", "nautilus", "📂"),
        ("Dolphin", "dolphin", "📂"), ("Nemo", "nemo", "📂"), ("Ranger", "ranger", "📂"),
        ("Less", "less", "👁️"), ("Bat", "bat", "👁️"), ("Cat", "cat", "👁️"),
        ("Eye of GNOME", "eog", "🖼️"), ("feh", "feh", "🖼️"),
        ("Gwenview", "gwenview", "🖼️"), ("Ristretto", "ristretto", "🖼️"),
        ("VLC", "vlc", "🎬"), ("MPV", "mpv", "🎬"), ("Celluloid", "celluloid", "🎬"),
        ("Evince", "evince", "📕"), ("Okular", "okular", "📕"),
        ("Zathura", "zathura", "📕"), ("LibreOffice", "libreoffice", "📄"),
        ("Firefox", "firefox", "🌐"), ("Chromium", "chromium", "🌐"),
        ("Google Chrome", "google-chrome-stable", "🌐"), ("Brave", "brave", "🌐"),
        ("GIMP", "gimp", "🎨"), ("Inkscape", "inkscape", "🎨"),
        ("Blender", "blender", "🎮"), ("Audacity", "audacity", "🎵"),
        ("Kdenlive", "kdenlive", "🎬"),
    ];

    let mut openers = vec![Opener {
        name: "System Default".to_string(),
        command: if cfg!(target_os = "macos") { "open" } else if cfg!(target_os = "windows") { "start" } else { "xdg-open" }.to_string(),
        icon: "🔧".to_string(),
    }];
    for (name, cmd, icon) in candidates {
        if command_exists(cmd) {
            openers.push(Opener { name: name.to_string(), command: cmd.to_string(), icon: icon.to_string() });
        }
    }
    openers
}

fn command_exists(cmd: &str) -> bool {
    std::process::Command::new("which").arg(cmd)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .status().map(|s| s.success()).unwrap_or(false)
}

fn read_dir(path: &PathBuf, show_hidden: bool) -> Vec<FileEntry> {
    let rd = match fs::read_dir(path) { Ok(rd) => rd, Err(_) => return vec![] };
    rd.filter_map(|entry| {
        let entry = entry.ok()?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') { return None; }
        let metadata = entry.metadata().ok();
        let is_symlink = entry.file_type().ok().map(|t| t.is_symlink()).unwrap_or(false);
        let is_dir = entry.file_type().ok().map(|t| t.is_dir()).unwrap_or(false);
        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        let modified = metadata.as_ref().and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        let is_readonly = metadata.as_ref().map(|m| m.permissions().readonly()).unwrap_or(false);
        #[cfg(unix)]
        let is_executable = {
            use std::os::unix::fs::PermissionsExt;
            metadata.as_ref().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
        };
        #[cfg(not(unix))]
        let is_executable = false;
        Some(FileEntry {
            name, path: entry.path(), is_dir, is_symlink, size, modified,
            is_hidden: entry.file_name().to_string_lossy().starts_with('.'),
            is_executable, is_readonly,
        })
    }).collect()
}