// src/explorer/actions.rs

use std::fs;
use std::path::PathBuf;

use super::state::{Confirm, Dialog, ExplorerState};

pub fn execute_dialog(state: &mut ExplorerState) -> Option<String> {
    let dialog = state.dialog.clone()?;
    let input = state.dialog_input.trim().to_string();
    if input.is_empty() { return Some("name cannot be empty".to_string()); }

    match dialog {
        Dialog::NewFile => {
            let path = state.cwd.join(&input);
            if path.exists() { Some(format!("'{}' already exists", input)) }
            else { match fs::write(&path, "") { Ok(_) => Some(format!("✓ created: {}", input)), Err(e) => Some(format!("✗ {}", e)) } }
        }
        Dialog::NewDir => {
            let path = state.cwd.join(&input);
            if path.exists() { Some(format!("'{}' already exists", input)) }
            else { match fs::create_dir_all(&path) { Ok(_) => Some(format!("✓ created: {}/", input)), Err(e) => Some(format!("✗ {}", e)) } }
        }
        Dialog::Rename => {
            if let Some(entry) = state.entries.get(state.selected) {
                let old = &entry.path;
                let new = state.cwd.join(&input);
                if new.exists() { Some(format!("'{}' already exists", input)) }
                else { match fs::rename(old, &new) { Ok(_) => Some(format!("✓ renamed → {}", input)), Err(e) => Some(format!("✗ {}", e)) } }
            } else { Some("nothing selected".to_string()) }
        }
    }
}

pub fn execute_confirm(state: &mut ExplorerState) -> Option<String> {
    let confirm = state.confirm.clone()?;
    match confirm {
        Confirm::Delete { ref name, is_dir } => {
            if let Some(entry) = state.entries.get(state.selected) {
                let path = &entry.path;
                let result = if is_dir { fs::remove_dir_all(path) } else { fs::remove_file(path) };
                match result { Ok(_) => Some(format!("✓ deleted: {}", name)), Err(e) => Some(format!("✗ {}", e)) }
            } else { Some("nothing selected".to_string()) }
        }
        Confirm::Overwrite { ref src, ref dest } => {
            let name = src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if src.is_dir() {
                match copy_dir_recursive(src, dest) { Ok(_) => Some(format!("✓ copied: {}", name)), Err(e) => Some(format!("✗ {}", e)) }
            } else {
                match fs::copy(src, dest) { Ok(_) => Some(format!("✓ copied: {}", name)), Err(e) => Some(format!("✗ {}", e)) }
            }
        }
    }
}

/// Paste a yanked file/dir. Handles name conflicts with _copy suffix.
pub fn paste(target_dir: &PathBuf, source: &PathBuf) -> String {
    let name = source.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string());
    let dest = target_dir.join(&name);

    let dest = if dest.exists() {
        let stem = dest.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = dest.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let mut i = 1;
        loop {
            let candidate = target_dir.join(format!("{}_copy{}{}", stem, i, ext));
            if !candidate.exists() { break candidate; }
            i += 1;
        }
    } else { dest };

    if source.is_dir() {
        match copy_dir_recursive(source, &dest) {
            Ok(_) => format!("✓ pasted: {} → {}", name, dest.file_name().unwrap_or_default().to_string_lossy()),
            Err(e) => format!("✗ copy failed: {}", e),
        }
    } else {
        match fs::copy(source, &dest) {
            Ok(bytes) => format!("✓ pasted: {} ({}) → {}", name, crate::utils::format_size(bytes),
                dest.file_name().unwrap_or_default().to_string_lossy()),
            Err(e) => format!("✗ copy failed: {}", e),
        }
    }
}

/// Move a file/dir to target directory
pub fn move_entry(target_dir: &PathBuf, source: &PathBuf) -> String {
    let name = source.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string());
    let dest = target_dir.join(&name);
    if dest.exists() { return format!("✗ '{}' already exists", name); }

    match fs::rename(source, &dest) {
        Ok(_) => format!("✓ moved: {}", name),
        Err(_) => {
            // Cross-device: copy then delete
            let copy_result = if source.is_dir() { copy_dir_recursive(source, &dest) }
                else { fs::copy(source, &dest).map(|_| ()) };
            match copy_result {
                Ok(_) => {
                    let _ = if source.is_dir() { fs::remove_dir_all(source) } else { fs::remove_file(source) };
                    format!("✓ moved: {} (cross-device)", name)
                }
                Err(e) => format!("✗ move failed: {}", e),
            }
        }
    }
}

/// Copy from external device (USB/phone) with progress-like status
pub fn copy_from_external(source: &PathBuf, target_dir: &PathBuf) -> String {
    let name = source.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let dest = target_dir.join(&name);

    if dest.exists() {
        let stem = dest.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext = dest.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
        let dest = target_dir.join(format!("{}_import{}", stem, ext));
        return do_copy(source, &dest, &name);
    }

    do_copy(source, &dest, &name)
}

fn do_copy(source: &PathBuf, dest: &PathBuf, name: &str) -> String {
    if source.is_dir() {
        match copy_dir_recursive(source, dest) {
            Ok(_) => {
                let total = dir_size(dest);
                format!("✓ imported: {} ({})", name, crate::utils::format_size(total))
            }
            Err(e) => format!("✗ import failed: {}", e),
        }
    } else {
        match fs::copy(source, dest) {
            Ok(bytes) => format!("✓ imported: {} ({})", name, crate::utils::format_size(bytes)),
            Err(e) => format!("✗ import failed: {}", e),
        }
    }
}

fn dir_size(path: &PathBuf) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = fs::read_dir(path) {
        for entry in rd.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                total += dir_size(&entry.path());
            } else {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
pub fn toggle_executable(path: &PathBuf) -> String {
    use std::os::unix::fs::PermissionsExt;
    match fs::metadata(path) {
        Ok(meta) => {
            let mut perms = meta.permissions();
            let mode = perms.mode();
            if mode & 0o111 != 0 {
                perms.set_mode(mode & !0o111);
                match fs::set_permissions(path, perms) { Ok(_) => "✓ removed +x".to_string(), Err(e) => format!("✗ {}", e) }
            } else {
                perms.set_mode(mode | 0o755);
                match fs::set_permissions(path, perms) { Ok(_) => "✓ added +x".to_string(), Err(e) => format!("✗ {}", e) }
            }
        }
        Err(e) => format!("✗ {}", e),
    }
}

#[cfg(not(unix))]
pub fn toggle_executable(_path: &PathBuf) -> String {
    "chmod not supported".to_string()
}