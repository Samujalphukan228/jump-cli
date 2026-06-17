use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

// ── CLI ────────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "jump", about = "Zero-config instant directory jumper", version = "0.2.0")]
struct Cli {
    /// Directory name, "-" to go back, or "seg1 seg2" for multi-segment
    query: Option<String>,

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

    /// Respect .gitignore files when pruning directories
    #[arg(long)]
    respect_gitignore: bool,

    /// Write resolved path to a file (used internally by shell wrapper)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show top jumped-to directories
    #[arg(long)]
    list: bool,

    /// Pin current directory or a path to a name: --pin <name> [path]
    #[arg(long, num_args = 1..=2, value_names = ["NAME", "PATH"])]
    pin: Option<Vec<String>>,

    /// Remove a pin by name
    #[arg(long, value_name = "NAME")]
    unpin: Option<String>,
}

// ── Data store ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
struct Store {
    frecency: HashMap<String, VisitStats>,
    pins: HashMap<String, String>,
    prev: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct VisitStats {
    visits: u64,
    last: u64,
}

impl Store {
    fn data_path() -> PathBuf {
        let base = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".local/share"));
        base.join("jump").join("data.json")
    }

    fn load() -> Self {
        let path = Self::data_path();
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self) {
        let path = Self::data_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&path, json);
        }
    }

    fn record_visit(&mut self, path: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = self.frecency.entry(path.to_string()).or_default();
        entry.visits += 1;
        entry.last = now;
    }

    fn frecency_score(&self, path: &str) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        match self.frecency.get(path) {
            None => 0.0,
            Some(s) => {
                let age_days = (now.saturating_sub(s.last)) as f64 / 86400.0;
                let decay = if age_days < 1.0 {
                    4.0
                } else if age_days < 7.0 {
                    2.0
                } else if age_days < 30.0 {
                    0.5
                } else {
                    0.25
                };
                s.visits as f64 * decay
            }
        }
    }
}

// ── Gitignore pruning ──────────────────────────────────────────────────────────

fn collect_gitignore_patterns(start: &PathBuf, root: &PathBuf) -> Vec<String> {
    let mut patterns = Vec::new();
    let mut current = start.as_path();
    loop {
        let gi = current.join(".gitignore");
        if gi.exists() {
            if let Ok(content) = fs::read_to_string(&gi) {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && !line.starts_with('#') {
                        patterns.push(line.trim_start_matches('/').to_string());
                    }
                }
            }
        }
        if current == root.as_path() {
            break;
        }
        match current.parent() {
            Some(p) => current = p,
            None => break,
        }
    }
    patterns
}

fn is_gitignored(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        let p = p.trim_end_matches('/');
        if p.starts_with('*') {
            name.ends_with(p.trim_start_matches('*'))
        } else {
            name == p
        }
    })
}

// ── Search ─────────────────────────────────────────────────────────────────────

fn match_rank(path: &PathBuf, query: &str) -> u8 {
    rank_name(
        &path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default(),
        query,
    )
}

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

fn format_tag(rank: u8) -> ColoredString {
    match rank {
        0 => " exact".green().dimmed(),
        1 => "prefix".cyan().dimmed(),
        2 => "  cont".normal().dimmed(),
        _ => " fuzzy".yellow().dimmed(),
    }
}

fn matches_fuzzy(name_lower: &str, query: &str) -> bool {
    let query_lower = query.to_lowercase();
    let initials: String = name_lower
        .split(|c: char| c == '-' || c == '_')
        .filter_map(|seg| seg.chars().next())
        .collect();
    if initials.contains(&query_lower) {
        return true;
    }
    let mut qchars = query_lower.chars().peekable();
    for ch in name_lower.chars() {
        if qchars.peek().map(|q| *q == ch).unwrap_or(false) {
            qchars.next();
        }
    }
    qchars.peek().is_none()
}

fn is_definitive(matches: &[PathBuf], query: &str) -> bool {
    matches.len() == 1
        && matches[0]
            .file_name()
            .map(|n| n.to_string_lossy().eq_ignore_ascii_case(query))
            .unwrap_or(false)
}

fn search(
    root: &PathBuf,
    query: &str,
    max_depth: usize,
    respect_gitignore: bool,
    store: &Store,
) -> Vec<PathBuf> {
    let parts: Vec<&str> = query.splitn(2, ' ').collect();
    let (path_filter, name_query) = if parts.len() == 2 {
        (Some(parts[0].to_lowercase()), parts[1])
    } else {
        (None, query)
    };

    let name_query_lower = name_query.to_lowercase();
    let gi_patterns = if respect_gitignore {
        collect_gitignore_patterns(root, root)
    } else {
        vec![]
    };

    let mut matches: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            if name.starts_with('.') || name == "node_modules" {
                return false;
            }
            if respect_gitignore && is_gitignored(&name, &gi_patterns) {
                return false;
            }
            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter(|e| {
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
            let name_matches = name.contains(&name_query_lower) || matches_fuzzy(&name, name_query);
            if !name_matches {
                return false;
            }
            if let Some(ref pf) = path_filter {
                let path_str = e.path().to_string_lossy().to_lowercase();
                if !path_str.contains(pf.as_str()) {
                    return false;
                }
            }
            true
        })
        .map(|e| e.path().to_path_buf())
        .filter(|p| p != root)
        .collect();

    matches.sort_by(|a, b| {
        let ra = match_rank(a, name_query);
        let rb = match_rank(b, name_query);
        if ra != rb {
            return ra.cmp(&rb);
        }
        let fa = store.frecency_score(&a.to_string_lossy());
        let fb = store.frecency_score(&b.to_string_lossy());
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });

    matches
}

// ── Pickers ────────────────────────────────────────────────────────────────────

fn read_choice() -> Option<String> {
    eprint!("{}", "Pick (0 or q to cancel): ".cyan().bold());
    io::stderr().flush().unwrap();
    let stdin = io::stdin();
    let line = stdin.lock().lines().next().unwrap().unwrap_or_default();
    let trimmed = line.trim().to_string();
    if trimmed == "0" || trimmed.eq_ignore_ascii_case("q") {
        None
    } else {
        Some(trimmed)
    }
}

fn pick_numbered(matches: &[PathBuf], label: Option<&str>, query: &str, store: &Store) -> PathBuf {
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
        let score = store.frecency_score(&path.to_string_lossy());
        let freq_tag = if score > 0.0 {
            format!(" ★{:.0}", score).yellow().to_string()
        } else {
            String::new()
        };
        eprintln!(
            "  {} [{}]{}  {}",
            format!("{}", i + 1).yellow().bold(),
            tag,
            freq_tag,
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
            let index: usize = if line.is_empty() { 1 } else { line.parse().unwrap_or(1) };
            let index = index.saturating_sub(1).min(matches.len() - 1);
            matches[index].clone()
        }
    }
}

fn pick_with_local_priority(local: &[PathBuf], other: &[PathBuf], query: &str, store: &Store) -> PathBuf {
    eprintln!("{} Found in current project:\n", "→".cyan().bold());
    for (i, path) in local.iter().enumerate() {
        let name_lower = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let tag = format_tag(rank_name(&name_lower, query));
        let score = store.frecency_score(&path.to_string_lossy());
        let freq_tag = if score > 0.0 {
            format!(" ★{:.0}", score).yellow().to_string()
        } else {
            String::new()
        };
        eprintln!(
            "  {} [{}]{}  {}",
            format!("{}", i + 1).yellow().bold(),
            tag,
            freq_tag,
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
            let index: usize = if line.is_empty() { 1 } else { line.parse().unwrap_or(1) };
            if index == search_everywhere_num {
                let all: Vec<PathBuf> = local.iter().chain(other.iter()).cloned().collect();
                pick_numbered(&all, Some("All matches:"), query, store)
            } else {
                let index = index.saturating_sub(1).min(local.len() - 1);
                local[index].clone()
            }
        }
    }
}

// ── --list ─────────────────────────────────────────────────────────────────────

fn cmd_list(store: &Store) {
    let mut entries: Vec<(&String, f64)> = store
        .frecency
        .iter()
        .map(|(k, _)| (k, store.frecency_score(k)))
        .filter(|(_, s)| *s > 0.0)
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    eprintln!("\n{}\n", "Top jumped directories:".bold());
    for (i, (path, score)) in entries.iter().take(20).enumerate() {
        let visits = store.frecency.get(*path).map(|s| s.visits).unwrap_or(0);
        eprintln!(
            "  {} {}  {} {}",
            format!("{:>2}.", i + 1).dimmed(),
            format!("★{:.0}", score).yellow(),
            path,
            format!("({} visits)", visits).dimmed()
        );
    }
    if !store.pins.is_empty() {
        eprintln!("\n{}\n", "Pins:".bold());
        let mut pins: Vec<(&String, &String)> = store.pins.iter().collect();
        pins.sort_by_key(|(k, _)| k.as_str());
        for (name, path) in pins {
            eprintln!("  {} → {}", format!("@{}", name).cyan().bold(), path);
        }
    }
    eprintln!();
}

// ── emit ───────────────────────────────────────────────────────────────────────

fn emit_target(target: &PathBuf, output: &Option<PathBuf>, store: &mut Store) {
    let target_str = target.to_string_lossy().to_string();
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());
    store.prev = cwd;
    store.record_visit(&target_str);
    store.save();
    if let Some(output_path) = output {
        fs::write(output_path, target.to_string_lossy().as_bytes())
            .expect("Failed to write output file");
    } else {
        println!("{}", target.display());
    }
}

// ── main ───────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let mut store = Store::load();

    if cli.list {
        cmd_list(&store);
        return;
    }

    if let Some(pin_args) = cli.pin {
        let name = &pin_args[0];
        let path = if pin_args.len() == 2 {
            PathBuf::from(&pin_args[1])
        } else {
            std::env::current_dir().expect("Could not get current dir")
        };
        let canonical = path.canonicalize().unwrap_or(path);
        store.pins.insert(name.clone(), canonical.to_string_lossy().to_string());
        store.save();
        eprintln!(
            "{} Pinned {} → {}",
            "✓".green().bold(),
            format!("@{}", name).cyan().bold(),
            canonical.display()
        );
        return;
    }

    if let Some(ref name) = cli.unpin {
        if store.pins.remove(name).is_some() {
            store.save();
            eprintln!("{} Unpinned @{}", "✓".green().bold(), name);
        } else {
            eprintln!("{} No pin named @{}", "✗".red().bold(), name);
        }
        return;
    }

    let query = match cli.query {
        Some(ref q) => q.as_str(),
        None => {
            eprintln!("{} Usage: jump <query>  or  jump -", "error:".red().bold());
            std::process::exit(1);
        }
    };

    // ── jump - ────────────────────────────────────────────────────────────────
    if query == "-" {
        match &store.prev {
            None => {
                eprintln!("{} No previous directory recorded", "✗".red().bold());
                std::process::exit(1);
            }
            Some(prev) => {
                let target = PathBuf::from(prev);
                if !target.exists() {
                    eprintln!(
                        "{} Previous directory no longer exists: {}",
                        "✗".red().bold(),
                        prev
                    );
                    std::process::exit(1);
                }
                eprintln!("{} {}", "→".green().bold(), target.display());
                emit_target(&target, &cli.output, &mut store);
                return;
            }
        }
    }

    // ── pin lookup ────────────────────────────────────────────────────────────
    if let Some(pinned_path) = store.pins.get(query) {
        let target = PathBuf::from(pinned_path);
        if target.exists() {
            eprintln!(
                "{} {} {}",
                "→".green().bold(),
                format!("[@{}]", query).cyan().dimmed(),
                target.display()
            );
            emit_target(&target, &cli.output, &mut store);
            return;
        } else {
            eprintln!(
                "{} Pin @{} points to missing path: {}",
                "!".yellow().bold(),
                query,
                target.display()
            );
        }
    }

    // ── search ────────────────────────────────────────────────────────────────
    let root = cli
        .root
        .unwrap_or_else(|| dirs::home_dir().expect("Could not find home directory"));

    if !root.exists() {
        eprintln!("{} Root path does not exist: {}", "error:".red().bold(), root.display());
        std::process::exit(1);
    }

    eprintln!(
        "{} {} {}",
        "jump".cyan().bold(),
        "searching for".dimmed(),
        query.yellow().bold()
    );

    let cwd = std::env::current_dir().unwrap_or_else(|_| root.clone());

    let local_matches = if cwd == root {
        vec![]
    } else {
        search(&cwd, query, cli.local_depth, cli.respect_gitignore, &store)
    };

    let (local_matches, other_matches) = if !cli.all && is_definitive(&local_matches, query) {
        (local_matches, vec![])
    } else {
        let local_set: HashSet<PathBuf> = local_matches.iter().cloned().collect();
        let other: Vec<PathBuf> = search(&root, query, cli.depth, cli.respect_gitignore, &store)
            .into_iter()
            .filter(|p| !local_set.contains(p))
            .collect();
        (local_matches, other)
    };

    let target = if local_matches.is_empty() && other_matches.is_empty() {
        eprintln!("{} No directory named '{}' found", "✗".red().bold(), query.yellow());
        std::process::exit(1);
    } else if local_matches.len() == 1 && other_matches.is_empty() {
        if match_rank(&local_matches[0], query) < 3 {
            local_matches[0].clone()
        } else {
            pick_numbered(&local_matches, None, query, &store)
        }
    } else if local_matches.is_empty() && other_matches.len() == 1 {
        if match_rank(&other_matches[0], query) < 3 {
            other_matches[0].clone()
        } else {
            pick_numbered(&other_matches, None, query, &store)
        }
    } else if other_matches.is_empty() {
        pick_numbered(&local_matches, None, query, &store)
    } else if local_matches.is_empty() {
        pick_numbered(&other_matches, None, query, &store)
    } else {
        pick_with_local_priority(&local_matches, &other_matches, query, &store)
    };

    eprintln!("{} {}", "→".green().bold(), target.display());
    emit_target(&target, &cli.output, &mut store);
}