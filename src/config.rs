// src/config.rs

use ratatui::style::Color;

// ── VOID design system — #121212 / #FAFAFA monochrome ───────────────────────

pub const BG: Color = Color::Rgb(18, 18, 18);
pub const BG_SURFACE: Color = Color::Rgb(24, 24, 24);
pub const BG_PANEL: Color = Color::Rgb(30, 30, 30);
pub const BG_CARD: Color = Color::Rgb(22, 22, 22);
pub const BG_INSET: Color = Color::Rgb(14, 14, 14);
pub const BG_SELECTED: Color = Color::Rgb(46, 46, 46);
pub const BG_OVERLAY: Color = Color::Rgb(12, 12, 12);

pub const TEXT: Color = Color::Rgb(250, 250, 250);
pub const TEXT_BRIGHT: Color = Color::Rgb(255, 255, 255);
pub const TEXT_DIM: Color = Color::Rgb(158, 158, 158);
pub const MUTED: Color = Color::Rgb(97, 97, 97);
pub const GHOST: Color = Color::Rgb(66, 66, 66);

pub const BORDER: Color = Color::Rgb(56, 56, 56);
pub const BORDER_DIM: Color = Color::Rgb(38, 38, 38);
pub const BORDER_ACTIVE: Color = Color::Rgb(250, 250, 250);
pub const RULE: Color = Color::Rgb(42, 42, 42);

pub const RAIL: Color = Color::Rgb(250, 250, 250);
pub const ACCENT: Color = TEXT;
pub const ACCENT_DIM: Color = TEXT_DIM;
pub const ACCENT_BRIGHT: Color = TEXT_BRIGHT;
pub const ACCENT_SOFT: Color = Color::Rgb(224, 224, 224);
pub const ACCENT_FAINT: Color = Color::Rgb(189, 189, 189);

// Legacy aliases
pub const CYAN: Color = ACCENT;
pub const CYAN_DIM: Color = ACCENT_DIM;
pub const CYAN_BRIGHT: Color = ACCENT_BRIGHT;
pub const YELLOW: Color = ACCENT_SOFT;
pub const GREEN: Color = ACCENT;
pub const RED: Color = ACCENT_BRIGHT;
pub const BLUE: Color = ACCENT;
pub const MAGENTA: Color = ACCENT_DIM;
pub const ORANGE: Color = ACCENT_SOFT;
pub const TEAL: Color = ACCENT_DIM;
pub const PINK: Color = ACCENT_DIM;
pub const LAVENDER: Color = ACCENT_DIM;
pub const PEACH: Color = ACCENT_SOFT;
pub const PHOSPHOR: Color = ACCENT;
pub const PHOSPHOR_DIM: Color = ACCENT_DIM;
pub const PHOSPHOR_BRIGHT: Color = ACCENT_BRIGHT;
pub const AMBER: Color = ACCENT_SOFT;

pub const SKIP_DIRS: &[&str] = &[
    "node_modules", ".next", "dist", "build", "__pycache__",
    ".pytest_cache", ".mypy_cache", ".ruff_cache", "venv", ".venv",
    "env", ".tox", ".eggs", "*.egg-info", "target", "debug",
    ".gradle", ".m2", ".idea", ".vscode", ".vs", ".git", ".svn",
    ".hg", ".Trash", ".Trash-1000", "$RECYCLE.BIN",
    "System Volume Information", ".npm", ".yarn", ".pnpm-store",
    ".cache", ".local", ".cargo", ".rustup", "coverage", ".coverage",
    "htmlcov", ".nyc_output", ".parcel-cache", ".turbo", ".angular",
    ".sass-cache", "bower_components", "vendor", ".bundle",
    ".terraform", ".serverless", "__MACOSX", ".debug", ".profile",
    ".docker", ".kube", "snap", ".snap",
];