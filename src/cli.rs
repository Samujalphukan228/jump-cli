// src/cli.rs

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "jump",
    about = "Instant directory jumper + file explorer",
    version = "0.4.0",
    disable_version_flag = true
)]
pub struct Cli {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub _version: (),

    /// Optional initial query or path for explorer
    pub query: Option<String>,

    #[arg(short, long)]
    pub root: Option<PathBuf>,

    #[arg(long, default_value = "4")]
    pub local_depth: usize,

    #[arg(long, default_value = "6")]
    pub depth: usize,

    #[arg(long)]
    pub all: bool,

    #[arg(long)]
    pub respect_gitignore: bool,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub list: bool,

    /// Open the file explorer TUI
    #[arg(short = 'e', long)]
    pub explore: bool,

    #[arg(long, num_args = 1..=2, value_names = ["NAME", "PATH"])]
    pub pin: Option<Vec<String>>,

    #[arg(long, value_name = "NAME")]
    pub unpin: Option<String>,
}