// src/ui/tui.rs

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::app::{App, SearchResults};
use crate::search::search;
use crate::store::{Store, VisitStats};
use crate::ui::components::draw_jump_flash;
use crate::ui::draw::draw;
use crate::ui::helpers::{tui_init, tui_restore};

fn spawn_search(
    query: String,
    root: PathBuf,
    cwd: PathBuf,
    local_depth: usize,
    depth: usize,
    respect_gitignore: bool,
    frecency: HashMap<String, VisitStats>,
    tx: Arc<Mutex<Option<SearchResults>>>,
    gen: u64,
    gc: Arc<Mutex<u64>>,
) {
    thread::spawn(move || {
        let local = if cwd == root {
            vec![]
        } else {
            search(
                &cwd,
                &query,
                local_depth,
                respect_gitignore,
                &frecency,
            )
        };
        if *gc.lock().unwrap() != gen {
            return;
        }
        *tx.lock().unwrap() = Some(SearchResults {
            query: query.clone(),
            matches: local.clone(),
            local_count: local.len(),
            searching: true,
            tick: 2,
        });
        let ls: HashSet<PathBuf> = local.iter().cloned().collect();
        let global: Vec<PathBuf> = search(&root, &query, depth, respect_gitignore, &frecency)
            .into_iter()
            .filter(|p| !ls.contains(p))
            .collect();
        if *gc.lock().unwrap() != gen {
            return;
        }
        let lc = local.len();
        let mut all = local;
        all.extend(global);
        *tx.lock().unwrap() = Some(SearchResults {
            query,
            matches: all,
            local_count: lc,
            searching: false,
            tick: 0,
        });
    });
}

fn resolve_selection(app: &App) -> Option<PathBuf> {
    let pin_view = !app.input.is_empty()
        && app.results.matches.is_empty()
        && !app.matching_pins().is_empty();
    let pin_only =
        app.input.is_empty() && !app.pins.is_empty() && app.results.matches.is_empty();
    if pin_view || pin_only {
        let pins = app.matching_pins();
        pins.get(app.selected)
            .map(|(_, path)| PathBuf::from(*path))
            .filter(|p| p.exists())
    } else {
        app.selected_path().cloned()
    }
}

pub fn run_tui(
    initial_query: Option<String>,
    root: PathBuf,
    local_depth: usize,
    depth: usize,
    respect_gitignore: bool,
    store: &Store,
) -> std::io::Result<Option<PathBuf>> {
    let mut terminal = tui_init()?;
    let cwd = std::env::current_dir().unwrap_or_else(|_| root.clone());
    let mut app = App::new(
        initial_query.as_deref().unwrap_or(""),
        store.pins.clone(),
        store.frecency.clone(),
    );

    let slot: Arc<Mutex<Option<SearchResults>>> = Arc::new(Mutex::new(None));
    let gc: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    if !app.input.is_empty() {
        app.results.searching = true;
        let g = {
            let mut c = gc.lock().unwrap();
            *c += 1;
            *c
        };
        spawn_search(
            app.input.clone(),
            root.clone(),
            cwd.clone(),
            local_depth,
            depth,
            respect_gitignore,
            store.frecency.clone(),
            Arc::clone(&slot),
            g,
            Arc::clone(&gc),
        );
    }

    let mut last_key = Instant::now();
    let debounce = Duration::from_millis(120);
    let mut pending = false;
    let mut tick: u64 = 0;
    let mut last_click: Option<(Instant, u16, u16, usize)> = None;

    let result = loop {
        {
            let mut s = slot.lock().unwrap();
            if let Some(nr) = s.take() {
                if nr.query == app.input {
                    app.results = nr;
                    app.clamp_selected();
                }
            }
        }

        tick += 1;
        app.results.tick = tick;
        terminal.draw(|f| draw(f, &mut app))?;

        if pending && last_key.elapsed() >= debounce {
            pending = false;
            if app.input.is_empty() {
                app.results = SearchResults::default();
            } else {
                app.results.searching = true;
                let g = {
                    let mut c = gc.lock().unwrap();
                    *c += 1;
                    *c
                };
                spawn_search(
                    app.input.clone(),
                    root.clone(),
                    cwd.clone(),
                    local_depth,
                    depth,
                    respect_gitignore,
                    store.frecency.clone(),
                    Arc::clone(&slot),
                    g,
                    Arc::clone(&gc),
                );
            }
        }

        if event::poll(Duration::from_millis(30))? {
            match event::read()? {
                Event::Mouse(mouse) => {
                    if let Some(target) = handle_mouse(&mut app, mouse, &mut last_click) {
                        terminal.draw(|f| draw_jump_flash(f, &target))?;
                        std::thread::sleep(Duration::from_millis(120));
                        break Some(target);
                    }
                }
                Event::Key(key) => match key.code {
                    KeyCode::Esc => break None,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        break None
                    }

                    KeyCode::Enter => {
                        if let Some(t) = resolve_selection(&app) {
                            terminal.draw(|f| draw_jump_flash(f, &t))?;
                            std::thread::sleep(Duration::from_millis(150));
                            break Some(t);
                        }
                    }

                    KeyCode::Up => app.move_up(),
                    KeyCode::Down => app.move_down(),
                    KeyCode::PageUp => app.page_up(10),
                    KeyCode::PageDown => app.page_down(10),
                    KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.move_up()
                    }
                    KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.move_down()
                    }
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.move_down()
                    }
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.move_up()
                    }

                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.clear_input();
                        pending = true;
                        last_key = Instant::now();
                    }
                    KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.delete_word();
                        pending = true;
                        last_key = Instant::now();
                    }
                    KeyCode::Backspace => {
                        app.backspace();
                        pending = true;
                        last_key = Instant::now();
                    }
                    KeyCode::Delete => {
                        app.delete_char();
                        pending = true;
                        last_key = Instant::now();
                    }
                    KeyCode::Left => app.cursor_left(),
                    KeyCode::Right => app.cursor_right(),
                    KeyCode::Home => app.cursor = 0,
                    KeyCode::End => app.cursor = app.input.len(),
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.cursor = 0
                    }
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.cursor = app.input.len()
                    }

                    KeyCode::Char(c)
                        if key.modifiers == KeyModifiers::NONE
                            || key.modifiers == KeyModifiers::SHIFT =>
                    {
                        app.insert_char(c);
                        pending = true;
                        last_key = Instant::now();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    };

    tui_restore(&mut terminal);
    Ok(result)
}

fn handle_mouse(
    app: &mut App,
    mouse: MouseEvent,
    last_click: &mut Option<(Instant, u16, u16, usize)>,
) -> Option<PathBuf> {
    match mouse.kind {
        MouseEventKind::ScrollUp => app.move_up(),
        MouseEventKind::ScrollDown => app.move_down(),
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(pos) = app.layout.cursor_at_search_click(
                mouse.column,
                mouse.row,
                app.input.len(),
            ) {
                app.cursor = pos;
                return None;
            }
            if let Some(idx) = app
                .layout
                .row_at(mouse.column, mouse.row, app.scroll_offset)
            {
                let count = if app.results.matches.is_empty()
                    && (!app.matching_pins().is_empty() || app.input.is_empty())
                {
                    app.matching_pins().len()
                } else {
                    app.results.matches.len()
                };
                if idx < count {
                    let now = Instant::now();
                    let double = last_click
                        .map(|(t, c, r, i)| {
                            t.elapsed() < Duration::from_millis(400)
                                && c == mouse.column
                                && r == mouse.row
                                && i == idx
                        })
                        .unwrap_or(false);
                    app.selected = idx;
                    app.clamp_selected();
                    if double {
                        *last_click = None;
                        return resolve_selection(app);
                    }
                    *last_click = Some((now, mouse.column, mouse.row, idx));
                }
            }
        }
        _ => {}
    }
    None
}