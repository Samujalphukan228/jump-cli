// src/filter.rs

use crate::config::SKIP_DIRS;
use std::fs;
use std::path::PathBuf;

pub fn should_skip_dir(name: &str) -> bool {
    if name.starts_with('.') {
        return true;
    }
    let name_lower = name.to_lowercase();
    for pattern in SKIP_DIRS {
        let p = pattern.to_lowercase();
        if p.starts_with('*') {
            if name_lower.ends_with(p.trim_start_matches('*')) {
                return true;
            }
        } else if p.starts_with('.') {
            if name == *pattern {
                return true;
            }
        } else if name_lower == p {
            return true;
        }
    }
    false
}

pub fn is_build_artifact(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    let path = entry.path();
    if name == "target" {
        if path.join("CACHEDIR.TAG").exists()
            || path.join(".rustc_info.json").exists()
            || path.join("release").exists()
            || path.join("debug").exists()
        {
            return true;
        }
    }
    if name == "node_modules" {
        return true;
    }
    if (name == "venv" || name == ".venv" || name == "env")
        && (path.join("pyvenv.cfg").exists() || path.join("bin/activate").exists())
    {
        return true;
    }
    if name == "debug" && path.join(".fingerprint").exists() {
        return true;
    }
    false
}

pub fn collect_gitignore_patterns(start: &PathBuf, root: &PathBuf) -> Vec<String> {
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

pub fn is_gitignored(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        let p = p.trim_end_matches('/');
        if p.starts_with('*') {
            name.ends_with(p.trim_start_matches('*'))
        } else {
            name == p
        }
    })
}