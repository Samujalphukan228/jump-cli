// src/config.rs

use ratatui::style::Color;

pub const CYAN: Color = Color::Rgb(137, 220, 235);
pub const CYAN_DIM: Color = Color::Rgb(69, 110, 118);
pub const CYAN_BRIGHT: Color = Color::Rgb(180, 240, 250);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const RED: Color = Color::Rgb(243, 139, 168);
pub const BLUE: Color = Color::Rgb(137, 180, 250);
pub const MAGENTA: Color = Color::Rgb(203, 166, 247);
pub const ORANGE: Color = Color::Rgb(250, 179, 135);
pub const TEAL: Color = Color::Rgb(148, 226, 213);
pub const PINK: Color = Color::Rgb(245, 194, 231);
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);
pub const PEACH: Color = Color::Rgb(250, 179, 135);

pub const BG: Color = Color::Rgb(24, 24, 37);
pub const BG_SURFACE: Color = Color::Rgb(30, 30, 46);
pub const BG_PANEL: Color = Color::Rgb(36, 39, 58);
pub const BG_SELECTED: Color = Color::Rgb(49, 50, 68);
pub const MUTED: Color = Color::Rgb(88, 91, 112);
pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const TEXT_DIM: Color = Color::Rgb(166, 173, 200);
pub const TEXT_BRIGHT: Color = Color::Rgb(235, 240, 255);
pub const BORDER: Color = Color::Rgb(69, 71, 90);
pub const BORDER_ACTIVE: Color = Color::Rgb(137, 180, 250);
pub const BORDER_DIM: Color = Color::Rgb(49, 50, 68);

pub const PHOSPHOR: Color = CYAN;
pub const PHOSPHOR_DIM: Color = CYAN_DIM;
pub const PHOSPHOR_BRIGHT: Color = CYAN_BRIGHT;
pub const AMBER: Color = YELLOW;

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