// src/main.rs

mod app;
mod cli;
mod config;
mod explorer;
mod filter;
mod search;
mod store;
mod ui;
mod utils;

use crate::cli::Cli;
use crate::explorer::run_explorer;
use crate::store::Store;
use crate::ui::dashboard::run_list_dashboard;
use crate::ui::tui::run_tui;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

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

fn main() {
    let cli = Cli::parse();
    let mut store = Store::load();

    // --explore
    if cli.explore {
        let start = cli
            .query
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().expect("Could not get cwd"));
        let start = start.canonicalize().unwrap_or(start);
        match run_explorer(&start) {
            Ok(Some(target)) => {
                emit_target(&target, &cli.output, &mut store);
            }
            Ok(None) => {
                eprintln!("cancelled.");
            }
            Err(e) => {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // --list
    if cli.list {
        if let Err(e) = run_list_dashboard(&store) {
            eprintln!("TUI error: {}", e);
        }
        return;
    }

    // --pin
    if let Some(pin_args) = cli.pin {
        let name = &pin_args[0];
        let path = if pin_args.len() == 2 {
            PathBuf::from(&pin_args[1])
        } else {
            std::env::current_dir().expect("Could not get current dir")
        };
        let canonical = path.canonicalize().unwrap_or(path);
        store
            .pins
            .insert(name.clone(), canonical.to_string_lossy().to_string());
        store.save();
        eprintln!("✓ pinned @{} → {}", name, canonical.display());
        return;
    }

    // --unpin
    if let Some(ref name) = cli.unpin {
        if store.pins.remove(name).is_some() {
            store.save();
            eprintln!("✓ unpinned @{}", name);
        } else {
            eprintln!("✗ no pin named @{}", name);
        }
        return;
    }

    // jump -
    if cli.query.as_deref() == Some("-") {
        match &store.prev {
            None => {
                eprintln!("✗ no previous directory");
                std::process::exit(1);
            }
            Some(prev) => {
                let target = PathBuf::from(prev);
                if !target.exists() {
                    eprintln!("✗ previous dir gone: {}", prev);
                    std::process::exit(1);
                }
                emit_target(&target, &cli.output, &mut store);
                return;
            }
        }
    }

    // Pin exact match — skip TUI
    if let Some(ref q) = cli.query {
        if let Some(pinned_path) = store.pins.get(q) {
            let target = PathBuf::from(pinned_path);
            if target.exists() {
                emit_target(&target, &cli.output, &mut store);
                return;
            }
        }
    }

    // Open TUI
    let root = cli
        .root
        .unwrap_or_else(|| dirs::home_dir().expect("No home directory"));

    if !root.exists() {
        eprintln!("error: root does not exist: {}", root.display());
        std::process::exit(1);
    }

    let target = match run_tui(
        cli.query,
        root,
        cli.local_depth,
        cli.depth,
        cli.respect_gitignore,
        &store,
    ) {
        Ok(Some(p)) => p,
        Ok(None) => {
            eprintln!("cancelled.");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    };

    emit_target(&target, &cli.output, &mut store);
}