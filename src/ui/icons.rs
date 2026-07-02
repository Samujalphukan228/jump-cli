// src/ui/icons.rs — Nerd Font glyphs via nerd-font-symbols + ASCII fallbacks
// Install a Nerd Font in your terminal (e.g. JetBrainsMono Nerd Font).

use nerd_font_symbols::{dev, md, seti};

pub const USE_NERD: bool = true;

#[inline]
fn pick(nerd: &'static str, fallback: &'static str) -> &'static str {
    if USE_NERD { nerd } else { fallback }
}

pub fn app_explore() -> &'static str { pick(seti::CUSTOM_FOLDER, "◆") }
pub fn home() -> &'static str { pick(seti::CUSTOM_HOME, "~") }
pub fn parent() -> &'static str { pick(md::MD_ARROW_UP, "↑") }
pub fn search() -> &'static str { pick(md::MD_MAGNIFY, "/") }
pub fn preview() -> &'static str { pick(md::MD_EYE_OUTLINE, "◎") }
pub fn drive() -> &'static str { pick(md::MD_USB, "▣") }
pub fn symlink() -> &'static str { pick(md::MD_LINK_VARIANT, "⤷") }
pub fn executable() -> &'static str { pick(md::MD_CONSOLE, "⚡") }
pub fn hidden() -> &'static str { pick(md::MD_EYE_OFF_OUTLINE, "·") }
pub fn folder_closed() -> &'static str { pick(seti::CUSTOM_FOLDER, "▸") }
pub fn folder_open() -> &'static str { pick(seti::CUSTOM_FOLDER_OPEN, "▾") }
pub fn file_plain() -> &'static str { pick(seti::CUSTOM_DEFAULT, "·") }

pub fn entry_icon(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        dir_icon(name)
    } else {
        file_icon_for_name(name)
    }
}

fn dir_icon(name: &str) -> &'static str {
    let n = name.to_lowercase();
    match n.as_str() {
        "src" | "source" | "lib" => pick(dev::DEV_RUST, "◈"),
        ".git" => pick(seti::CUSTOM_FOLDER_GIT, "◎"),
        ".github" => pick(seti::CUSTOM_FOLDER_GITHUB, "◎"),
        "node_modules" | "vendor" | "target" => pick(seti::CUSTOM_FOLDER_NPM, "▣"),
        "docs" | "doc" => pick(md::MD_BOOK_OPEN_PAGE_VARIANT, "≡"),
        "test" | "tests" => pick(md::MD_TEST_TUBE, "◇"),
        "config" | "conf" | ".config" => pick(seti::CUSTOM_FOLDER_CONFIG, "⚙"),
        "dist" | "build" | "out" => pick(md::MD_PACKAGE_VARIANT_CLOSED, "▣"),
        _ if name.starts_with('.') => pick(md::MD_FOLDER_HIDDEN, "·"),
        _ => folder_closed(),
    }
}

fn file_icon_for_name(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if let Some(icon) = match lower.as_str() {
        "cargo.toml" => Some(pick(seti::CUSTOM_TOML, "◈")),
        "cargo.lock" | "package-lock.json" | "yarn.lock" => Some(pick(md::MD_LOCK, "◌")),
        "package.json" => Some(pick(md::MD_PACKAGE_VARIANT_CLOSED, "◌")),
        "readme.md" | "readme" => Some(pick(md::MD_INFORMATION_OUTLINE, "≡")),
        "makefile" => Some(pick(md::MD_HAMMER, "◌")),
        _ => None,
    } {
        return icon;
    }
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "rs" => pick(dev::DEV_RUST, "◈"),
        "py" => pick(dev::DEV_PYTHON, "◈"),
        "js" | "mjs" | "cjs" => pick(dev::DEV_JAVASCRIPT, "◈"),
        "ts" | "tsx" => pick(dev::DEV_TYPESCRIPT, "◈"),
        "jsx" => pick(dev::DEV_REACT, "◈"),
        "go" => pick(seti::CUSTOM_GO, "◈"),
        "json" | "jsonc" => pick(md::MD_CODE_JSON, "◌"),
        "yaml" | "yml" => pick(md::MD_FILE_CODE, "◌"),
        "toml" => pick(seti::CUSTOM_TOML, "◌"),
        "md" | "mdx" => pick(md::MD_LANGUAGE_MARKDOWN, "≡"),
        "sh" | "bash" | "zsh" => pick(md::MD_CONSOLE, "◌"),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => pick(md::MD_FILE_IMAGE, "□"),
        "mp4" | "mkv" | "mov" | "webm" => pick(md::MD_FILE_VIDEO, "▷"),
        "mp3" | "wav" | "flac" | "ogg" => pick(md::MD_FILE_MUSIC, "♪"),
        "zip" | "tar" | "gz" | "7z" | "rar" => pick(md::MD_FOLDER_ZIP, "▣"),
        "lock" => pick(md::MD_LOCK, "◌"),
        "log" => pick(md::MD_TEXT_BOX_OUTLINE, "≡"),
        _ => file_plain(),
    }
}