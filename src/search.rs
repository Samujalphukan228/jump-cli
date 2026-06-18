// src/search.rs

use crate::filter::{collect_gitignore_patterns, is_build_artifact, is_gitignored, should_skip_dir};
use crate::store::VisitStats;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub fn match_rank(path: &PathBuf, query: &str) -> u8 {
    rank_name(
        &path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default(),
        query,
    )
}

pub fn rank_name(name_lower: &str, query: &str) -> u8 {
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

pub fn matches_fuzzy(name_lower: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
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

pub fn search(
    root: &PathBuf,
    query: &str,
    max_depth: usize,
    respect_gitignore: bool,
    frecency: &HashMap<String, VisitStats>,
) -> Vec<PathBuf> {
    if query.is_empty() {
        return vec![];
    }

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

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let frecency_score = |path: &str| -> f64 {
        match frecency.get(path) {
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
    };

    let mut matches: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !e.file_type().is_dir() {
                return false;
            }
            let name = e.file_name().to_string_lossy();
            if should_skip_dir(&name) {
                return false;
            }
            if is_build_artifact(e) {
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
            let name = e.file_name().to_string_lossy().to_lowercase();
            let name_matches =
                name.contains(&name_query_lower) || matches_fuzzy(&name, name_query);
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
        let fa = frecency_score(&a.to_string_lossy());
        let fb = frecency_score(&b.to_string_lossy());
        fb.partial_cmp(&fa).unwrap_or(std::cmp::Ordering::Equal)
    });

    matches
}