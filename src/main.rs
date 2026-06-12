use clap::Parser;
use colored::*;
use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "jump", about = "Zero-config instant directory jumper", version = "0.1.0")]
struct Cli {
    query: String,

    #[arg(short, long)]
    root: Option<PathBuf>,

    /// Search depth from cwd (default 4)
    #[arg(long, default_value = "4")]
    local_depth: usize,

    /// Search depth from home (default 6)
    #[arg(long, default_value = "6")]
    depth: usize,

    /// Skip lazy home-search optimisation and always search everywhere
    #[arg(long)]
    all: bool,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let root = cli
        .root
        .unwrap_or_else(|| dirs::home_dir().expect("Could not find home directory"));

    if !root.exists() {
        eprintln!(
            "{} Root path does not exist: {}",
            "error:".red().bold(),
            root.display()
        );
        std::process::exit(1);
    }

    eprintln!(
        "{} {} {}",
        "jump".cyan().bold(),
        "searching for".dimmed(),
        cli.query.yellow().bold()
    );

    let cwd = std::env::current_dir().unwrap_or_else(|_| root.clone());

    // Search locally first; only search home if local results aren't definitive.
    // Skip the local search entirely when cwd == root (they'd be the same walk).
    let local_matches = if cwd == root {
        vec![]
    } else {
        search(&cwd, &cli.query, cli.local_depth)
    };

    let (local_matches, other_matches) =
        if !cli.all && is_definitive(&local_matches, &cli.query) {
            (local_matches, vec![])
        } else {
            // Deduplicate with a HashSet to avoid O(n²) contains checks.
            let local_set: HashSet<PathBuf> = local_matches.iter().cloned().collect();
            let other: Vec<PathBuf> = search(&root, &cli.query, cli.depth)
                .into_iter()
                .filter(|p| !local_set.contains(p))
                .collect();
            (local_matches, other)
        };

    let target = if local_matches.is_empty() && other_matches.is_empty() {
        eprintln!(
            "{} No directory named '{}' found",
            "✗".red().bold(),
            cli.query.yellow()
        );
        std::process::exit(1);
    } else if local_matches.len() == 1 && other_matches.is_empty() {
        // Only auto-jump on exact/prefix/contains; fuzzy still prompts.
        if match_rank(&local_matches[0], &cli.query) < 3 {
            local_matches[0].clone()
        } else {
            pick_numbered(&local_matches, None, &cli.query)
        }
    } else if local_matches.is_empty() && other_matches.len() == 1 {
        if match_rank(&other_matches[0], &cli.query) < 3 {
            other_matches[0].clone()
        } else {
            pick_numbered(&other_matches, None, &cli.query)
        }
    } else if other_matches.is_empty() {
        pick_numbered(&local_matches, None, &cli.query)
    } else if local_matches.is_empty() {
        pick_numbered(&other_matches, None, &cli.query)
    } else {
        pick_with_local_priority(&local_matches, &other_matches, &cli.query)
    };

    eprintln!("{} {}", "→".green().bold(), target.display());

    if let Some(output_path) = cli.output {
        fs::write(&output_path, target.to_string_lossy().as_bytes())
            .expect("Failed to write output file");
    } else {
        println!("{}", target.display());
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A single exact-name local match is definitive — skip scanning home.
fn is_definitive(matches: &[PathBuf], query: &str) -> bool {
    matches.len() == 1
        && matches[0]
            .file_name()
            .map(|n| n.to_string_lossy().eq_ignore_ascii_case(query))
            .unwrap_or(false)
}

/// Sort key: 0 = exact, 1 = prefix, 2 = contains, 3 = fuzzy.
fn match_rank(path: &PathBuf, query: &str) -> u8 {
    rank_name(
        &path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default(),
        query,
    )
}

/// Same as match_rank but operates on an already-lowercased name string.
fn rank_name(name_lower: &str, query: &str) -> u8 {
    let q = query.to_lowercase();
    if *name_lower == q {
        0
    } else if name_lower.starts_with(&q) {
        1
    } else if name_lower.contains(&q) {
        2
    } else {
        3
    }
}

/// Render a coloured match-type tag for the picker UI.
fn format_tag(rank: u8) -> ColoredString {
    match rank {
        0 => " exact".green().dimmed(),
        1 => "prefix".cyan().dimmed(),
        2 => "  cont".normal().dimmed(),
        _ => " fuzzy".yellow().dimmed(),
    }
}

/// Initialism match: "nxb" → "nexxupp-backend" (first char of each segment).
/// Subsequence fallback: "nxb" → "noxbuild".
fn matches_fuzzy(name_lower: &str, query: &str) -> bool {
    let query_lower = query.to_lowercase();

    let initials: String = name_lower
        .split(|c: char| c == '-' || c == '_')
        .filter_map(|seg| seg.chars().next())
        .collect();

    if initials.contains(&query_lower) {
        return true;
    }

    // Subsequence: every query char must appear in order within name.
    let mut qchars = query_lower.chars().peekable();
    for ch in name_lower.chars() {
        if qchars.peek().map(|q| *q == ch).unwrap_or(false) {
            qchars.next();
        }
    }
    qchars.peek().is_none()
}

// ---------------------------------------------------------------------------
// Directory search
// ---------------------------------------------------------------------------

fn search(root: &PathBuf, query: &str, max_depth: usize) -> Vec<PathBuf> {
    let query_lower = query.to_lowercase();

    let mut matches: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden dirs and known build/tool noise.
            // NOTE: "target" is intentionally NOT pruned here because it's a
            // common real directory name (e.g. "target-corp"). We skip it only
            // in the results filter below if it looks like a Rust build dir.
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "node_modules"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| {
            // Skip Rust build directories: named "target" AND contain a
            // ".rustc_info.json" or "CACHEDIR.TAG" file as a heuristic.
            let name = e.file_name().to_string_lossy();
            if name == "target" {
                let p = e.path();
                if p.join("CACHEDIR.TAG").exists() || p.join(".rustc_info.json").exists() {
                    return false;
                }
            }
            true
        })
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            name.contains(&query_lower) || matches_fuzzy(&name, query)
        })
        .map(|e| e.path().to_path_buf())
        .filter(|p| p != root)
        .collect();

    matches.sort_by_key(|p| match_rank(p, query));
    matches
}

// ---------------------------------------------------------------------------
// Interactive pickers
// ---------------------------------------------------------------------------

/// Read a line from stdin. Returns `None` if the user types "0" or "q" (cancel).
fn read_choice() -> Option<String> {
    eprint!("{}", "Pick (0 or q to cancel): ".cyan().bold());
    io::stderr().flush().unwrap();

    let stdin = io::stdin();
    let line = stdin
        .lock()
        .lines()
        .next()
        .unwrap()
        .unwrap_or_default();
    let trimmed = line.trim().to_string();

    if trimmed == "0" || trimmed.eq_ignore_ascii_case("q") {
        None
    } else {
        Some(trimmed)
    }
}

fn pick_numbered(matches: &[PathBuf], label: Option<&str>, query: &str) -> PathBuf {
    if let Some(l) = label {
        eprintln!("\n{}", l.dimmed());
    } else {
        eprintln!(
            "{} Multiple matches for '{}':\n",
            "→".cyan().bold(),
            query.yellow()
        );
    }

    for (i, path) in matches.iter().enumerate() {
        let name_lower = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let tag = format_tag(rank_name(&name_lower, query));
        eprintln!(
            "  {} [{}]  {}",
            format!("{}", i + 1).yellow().bold(),
            tag,
            path.display()
        );
    }

    eprintln!();

    match read_choice() {
        None => {
            eprintln!("{}", "Cancelled.".dimmed());
            std::process::exit(0);
        }
        Some(line) => {
            // Blank input → default to 1
            let index: usize = if line.is_empty() {
                1
            } else {
                line.parse().unwrap_or(1)
            };
            let index = index.saturating_sub(1).min(matches.len() - 1);
            matches[index].clone()
        }
    }
}

fn pick_with_local_priority(local: &[PathBuf], other: &[PathBuf], query: &str) -> PathBuf {
    eprintln!("{} Found in current project:\n", "→".cyan().bold());

    for (i, path) in local.iter().enumerate() {
        let name_lower = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let tag = format_tag(rank_name(&name_lower, query));
        eprintln!(
            "  {} [{}]  {}",
            format!("{}", i + 1).yellow().bold(),
            tag,
            path.display()
        );
    }

    let search_everywhere_num = local.len() + 1;
    eprintln!(
        "  {}        {}",
        format!("{}", search_everywhere_num).dimmed(),
        "Search everywhere".dimmed()
    );

    eprintln!();

    match read_choice() {
        None => {
            eprintln!("{}", "Cancelled.".dimmed());
            std::process::exit(0);
        }
        Some(line) => {
            let index: usize = if line.is_empty() {
                1
            } else {
                line.parse().unwrap_or(1)
            };

            if index == search_everywhere_num {
                // Merge local + other, preserving rank order within each group.
                let all: Vec<PathBuf> = local.iter().chain(other.iter()).cloned().collect();
                pick_numbered(&all, Some("All matches:"), query)
            } else {
                let index = index.saturating_sub(1).min(local.len() - 1);
                local[index].clone()
            }
        }
    }
}