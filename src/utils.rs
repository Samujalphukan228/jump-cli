// src/utils.rs

use crate::config::*;
use ratatui::style::Color;
use std::path::PathBuf;

pub fn shorten_path(path: &PathBuf) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~/{}", rel.display());
        }
    }
    path.to_string_lossy().to_string()
}

pub fn rank_badge(rank: u8) -> (&'static str, Color) {
    match rank {
        0 => ("exact ", TEXT_BRIGHT),
        1 => ("prefix", TEXT),
        2 => ("match ", TEXT_DIM),
        _ => ("fuzzy ", MUTED),
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn format_time(secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let diff = now.saturating_sub(secs);
    if diff < 60 {
        "now".to_string()
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else if diff < 86400 * 30 {
        format!("{}d", diff / 86400)
    } else {
        format!("{}mo", diff / (86400 * 30))
    }
}

/// File/directory icons using clean emoji
pub fn file_icon(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        let n = name.to_lowercase();
        return match n.as_str() {
            "src" | "source" | "lib" => "⚡",
            "test" | "tests" | "__tests__" | "spec" | "specs" => "🧪",
            "docs" | "doc" | "documentation" => "📚",
            "config" | "conf" | ".config" => "🔧",
            "build" | "dist" | "out" | "output" => "📦",
            "assets" | "static" | "public" | "media" => "🎨",
            "scripts" | "bin" | "tools" => "⚙️",
            ".git" => "🔗",
            ".github" => "🔗",
            "node_modules" => "📦",
            ".vscode" => "🔧",
            ".idea" => "🔧",
            "target" => "📦",
            "vendor" => "📦",
            "pkg" | "packages" => "📦",
            "components" => "🧩",
            "pages" | "views" => "🌐",
            "styles" | "css" => "🎨",
            "images" | "img" | "icons" => "🖼️",
            "fonts" => "🔤",
            "hooks" => "🔌",
            "utils" | "helpers" => "🔧",
            "models" | "schemas" => "🗄️",
            "api" | "routes" => "🔌",
            "middleware" => "⚙️",
            "migrations" => "🗄️",
            "templates" | "layouts" => "📄",
            "plugins" | "addons" | "extensions" => "🧩",
            "data" | "db" | "database" => "🗄️",
            "logs" | "log" => "📋",
            "tmp" | "temp" | "cache" => "📋",
            "backup" | "backups" => "📦",
            "downloads" => "📥",
            "uploads" => "📤",
            "i18n" | "locales" | "lang" | "translations" => "🌐",
            "types" | "interfaces" => "💠",
            "services" => "⚙️",
            "controllers" => "⚙️",
            "examples" | "samples" | "demo" => "📖",
            _ => "📁",
        };
    }

    // Exact filename matches
    match name.to_lowercase().as_str() {
        "makefile" | "gnumakefile" | "justfile" => return "🏗️",
        "dockerfile" => return "🐳",
        "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml" => {
            return "🐳"
        }
        "license" | "licence" | "license.md" | "licence.md" => return "📜",
        "readme" | "readme.md" | "readme.txt" | "readme.rst" => return "📖",
        "changelog" | "changelog.md" => return "📋",
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "🚫",
        ".dockerignore" => return "🚫",
        ".editorconfig" => return "🔧",
        ".env" | ".env.local" | ".env.production" | ".env.development" => return "🔐",
        ".eslintrc" | ".eslintrc.js" | ".eslintrc.json" | ".eslintrc.yml" => return "🔧",
        ".prettierrc" | ".prettierrc.js" | ".prettierrc.json" => return "🔧",
        "cargo.toml" => return "🦀",
        "cargo.lock" => return "🔒",
        "package.json" => return "📋",
        "package-lock.json" => return "🔒",
        "yarn.lock" => return "🔒",
        "pnpm-lock.yaml" => return "🔒",
        "tsconfig.json" | "tsconfig.node.json" | "tsconfig.app.json" => return "💠",
        "go.mod" | "go.sum" => return "🔷",
        "gemfile" | "gemfile.lock" => return "💎",
        "rakefile" => return "💎",
        "requirements.txt" | "pipfile" | "pipfile.lock" => return "🐍",
        "setup.py" | "setup.cfg" | "pyproject.toml" => return "🐍",
        "webpack.config.js" | "webpack.config.ts" => return "📦",
        "vite.config.js" | "vite.config.ts" => return "⚡",
        "tailwind.config.js" | "tailwind.config.ts" => return "🎨",
        "next.config.js" | "next.config.mjs" | "next.config.ts" => return "🌐",
        "nuxt.config.js" | "nuxt.config.ts" => return "🌐",
        "jest.config.js" | "jest.config.ts" => return "🧪",
        ".travis.yml" => return "⚙️",
        "netlify.toml" | "vercel.json" => return "🌐",
        "flake.nix" | "flake.lock" | "shell.nix" | "default.nix" => return "❄️",
        "cmakelists.txt" => return "🏗️",
        "build.gradle" | "build.gradle.kts" | "settings.gradle" => return "🏗️",
        "pom.xml" => return "☕",
        "procfile" => return "⚙️",
        _ => {}
    }

    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        // Languages
        "rs" => "🦀",
        "py" | "pyw" | "pyx" | "pxd" | "pyi" => "🐍",
        "js" | "mjs" | "cjs" => "⚡",
        "ts" | "mts" | "cts" => "💠",
        "jsx" => "⚛️",
        "tsx" => "⚛️",
        "go" => "🔷",
        "c" => "⚙️",
        "h" => "⚙️",
        "cpp" | "cc" | "cxx" | "c++" => "⚙️",
        "hpp" | "hxx" | "h++" => "⚙️",
        "cs" | "csx" => "💠",
        "java" => "☕",
        "kt" | "kts" => "💠",
        "swift" => "🔶",
        "rb" | "erb" => "💎",
        "php" => "🐘",
        "lua" => "🌙",
        "zig" => "⚡",
        "nim" | "nims" => "👑",
        "hs" | "lhs" => "⚗️",
        "ex" | "exs" | "heex" | "leex" => "⚗️",
        "erl" | "hrl" => "⚗️",
        "r" | "rmd" | "rdata" | "rds" => "📊",
        "jl" => "📊",
        "dart" => "🎯",
        "scala" | "sc" | "sbt" => "🔴",
        "clj" | "cljs" | "cljc" | "edn" => "🔮",
        "ml" | "mli" => "🔶",
        "fs" | "fsx" | "fsi" => "💠",
        "pl" | "pm" | "pod" => "🐪",
        "asm" | "s" => "⚙️",
        "v" | "vh" | "sv" | "svh" => "⚙️",

        // Shell
        "sh" | "bash" => "🐚",
        "zsh" => "🐚",
        "fish" => "🐚",
        "ps1" | "psm1" | "psd1" => "🐚",
        "bat" | "cmd" => "🐚",

        // Web
        "html" | "htm" | "xhtml" => "🌐",
        "css" => "🎨",
        "scss" | "sass" => "🎨",
        "less" => "🎨",
        "vue" => "💚",
        "svelte" => "🔥",
        "astro" => "🚀",

        // Data / Config
        "json" | "jsonc" | "json5" => "📋",
        "yaml" | "yml" => "📋",
        "toml" => "📋",
        "xml" | "xsl" | "xslt" => "📄",
        "csv" | "tsv" => "📊",
        "ini" | "cfg" | "conf" => "🔧",
        "sql" | "sqlite" | "db" => "🗄️",
        "graphql" | "gql" => "🔮",
        "proto" | "protobuf" => "📋",
        "env" => "🔐",

        // Documentation
        "md" | "mdx" => "📝",
        "rst" => "📝",
        "txt" => "📄",
        "pdf" => "📕",
        "doc" | "docx" => "📄",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "tex" | "latex" | "bib" => "📝",
        "org" => "📝",

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" => "🖼️",
        "ico" | "icns" => "🖼️",
        "svg" => "🖼️",
        "webp" | "avif" | "heif" | "heic" => "🖼️",
        "psd" => "🎨",
        "ai" => "🎨",
        "sketch" | "fig" | "figma" => "🎨",
        "blend" | "3ds" | "fbx" | "stl" => "🎮",

        // Audio
        "mp3" | "wav" | "wave" | "flac" | "aac" | "m4a" => "🎵",
        "ogg" | "oga" | "opus" | "wma" => "🎵",
        "mid" | "midi" => "🎵",

        // Video
        "mp4" | "m4v" | "mkv" | "avi" | "mov" => "🎬",
        "wmv" | "webm" | "flv" => "🎬",

        // Archives
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "tbz2" => "📦",
        "xz" | "txz" | "7z" | "rar" | "zst" | "zstd" => "📦",
        "deb" | "rpm" | "dmg" | "iso" => "📦",
        "pkg" | "apk" | "msi" => "📦",

        // Binary / Executable
        "bin" | "exe" => "⚙️",
        "dll" | "so" | "dylib" => "🔌",
        "o" | "a" | "lib" => "⚙️",
        "wasm" | "wat" => "🧩",
        "appimage" | "snap" | "flatpak" => "📦",

        // Fonts
        "ttf" | "otf" | "woff" | "woff2" | "eot" => "🔤",

        // Security / Keys
        "key" | "pem" | "pub" | "gpg" | "pgp" => "🔐",
        "crt" | "cer" | "ca" => "🔐",
        "p12" | "pfx" | "jks" => "🔐",
        "sig" | "asc" => "🔐",

        // Nix
        "nix" => "❄️",

        // Misc
        "lock" => "🔒",
        "log" => "📋",
        "bak" | "swp" | "swo" | "tmp" => "📋",
        "patch" | "diff" => "📝",
        "map" => "📋",

        _ => "📄",
    }
}

/// Color for the file icon — grayscale only
pub fn file_icon_color(name: &str, is_dir: bool) -> Color {
    if is_dir {
        let n = name.to_lowercase();
        return match n.as_str() {
            "node_modules" | "vendor" | "target" | "build" | "dist" | ".git" | ".github" => MUTED,
            "src" | "source" | "lib" | "test" | "tests" | "docs" | "doc" => TEXT_BRIGHT,
            _ if name.starts_with('.') => TEXT_DIM,
            _ => TEXT,
        };
    }

    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "lock" | "log" | "bak" | "tmp" | "swp" => MUTED,
        "md" | "mdx" | "rst" | "txt" => TEXT_DIM,
        _ => TEXT_DIM,
    }
}