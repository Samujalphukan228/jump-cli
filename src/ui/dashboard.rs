// src/ui/dashboard.rs

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
    Frame,
};
use std::path::PathBuf;
use std::time::Duration;

use crate::config::*;
use crate::store::Store;
use crate::ui::helpers::{tui_init, tui_restore};
use crate::utils::shorten_path;

pub fn run_list_dashboard(store: &Store) -> std::io::Result<()> {
    let mut terminal = tui_init()?;
    loop {
        terminal.draw(|f| draw_list(f, store))?;
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                }
            }
        }
    }
    tui_restore(&mut terminal);
    Ok(())
}

fn draw_list(f: &mut Frame, store: &Store) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)]).split(area);

    f.render_widget(Block::default().borders(Borders::BOTTOM)
        .border_style(Style::default().fg(PHOSPHOR_DIM))
        .style(Style::default().bg(BG_PANEL)), chunks[0]);

    let ti = chunks[0].inner(Margin { horizontal: 2, vertical: 1 });
    f.render_widget(Paragraph::new(Line::from(vec![
        Span::styled("⚡ jump ", Style::default().fg(PHOSPHOR).add_modifier(Modifier::BOLD)),
        Span::styled("— history  ", Style::default().fg(MUTED)),
        Span::styled(format!("{} tracked", store.frecency.len()), Style::default().fg(PHOSPHOR_DIM)),
    ])), ti);

    let has_pins = !store.pins.is_empty();
    let mc = if has_pins {
        Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)]).split(chunks[1])
    } else {
        Layout::default().direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)]).split(chunks[1])
    };

    let mut entries: Vec<(&String, f64)> = store.frecency.iter()
        .map(|(k, _)| (k, store.frecency_score(k)))
        .filter(|(_, s)| *s > 0.0).collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let items: Vec<ListItem> = entries.iter().take(30).enumerate().map(|(i, (path, score))| {
        let visits = store.frecency.get(*path).map(|s| s.visits).unwrap_or(0);
        let short = shorten_path(&PathBuf::from(path.as_str()));
        let pb = PathBuf::from(path.as_str());
        let name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string());
        let parent = short.rfind('/').map(|p| format!("{}/", &short[..p])).unwrap_or_default();
        let max = entries.first().map(|(_, s)| *s).unwrap_or(1.0).max(1.0);
        let bl = ((score / max) * 10.0) as usize;
        let bar = "█".repeat(bl) + &"░".repeat(10 - bl);
        ListItem::new(Line::from(vec![
            Span::styled(format!("{:>2}  ", i + 1), Style::default().fg(PHOSPHOR_DIM)),
            Span::styled(bar, Style::default().fg(PHOSPHOR_DIM)),
            Span::styled(format!(" ★{:<5.0}", score), Style::default().fg(AMBER)),
            Span::styled(format!(" {:>4}v  ", visits), Style::default().fg(MUTED)),
            Span::styled(parent, Style::default().fg(MUTED)),
            Span::styled(name, Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]))
    }).collect();

    let fb = Block::default().borders(Borders::ALL)
        .border_style(Style::default().fg(PHOSPHOR_DIM))
        .title(Span::styled(" ★ frecency ", Style::default().fg(PHOSPHOR)))
        .padding(Padding::horizontal(1)).style(Style::default().bg(BG_PANEL));

    if items.is_empty() {
        f.render_widget(Paragraph::new(vec![Line::from(""),
            Line::from(vec![Span::styled("  no history yet", Style::default().fg(MUTED))])
        ]).block(fb), mc[0]);
    } else {
        f.render_widget(List::new(items).block(fb), mc[0]);
    }

    if has_pins {
        let mut pins: Vec<(&String, &String)> = store.pins.iter().collect();
        pins.sort_by_key(|(k, _)| k.as_str());
        let pi: Vec<ListItem> = pins.iter().map(|(name, path)| {
            let short = shorten_path(&PathBuf::from(path.as_str()));
            ListItem::new(Line::from(vec![
                Span::styled(format!("@{:<12}", name), Style::default().fg(PHOSPHOR).add_modifier(Modifier::BOLD)),
                Span::styled("→ ", Style::default().fg(PHOSPHOR_DIM)),
                Span::styled(short, Style::default().fg(TEXT)),
            ]))
        }).collect();
        let pb = Block::default().borders(Borders::ALL)
            .border_style(Style::default().fg(PHOSPHOR_DIM))
            .title(Span::styled(" ⬡ pins ", Style::default().fg(PHOSPHOR)))
            .padding(Padding::horizontal(1)).style(Style::default().bg(BG_PANEL));
        f.render_widget(List::new(pi).block(pb), mc[1]);
    }
}