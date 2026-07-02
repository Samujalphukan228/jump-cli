# jump-cli

**cd at the speed of thought. browse files without leaving the terminal.**

[![version](https://img.shields.io/badge/version-0.4.0-lightgrey?style=flat-square)](https://github.com/Samujalphukan228/jump-cli/releases)
[![license](https://img.shields.io/badge/license-MIT-lightgrey?style=flat-square)](LICENSE)

Two tools, one binary:

- **`jump`** — fuzzy directory search with a live TUI
- **`exp`** — terminal file explorer with preview, copy/paste, and mouse support

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh
source ~/.bashrc   # or ~/.zshrc
```

---

## Install

**One-liner** (installs Rust if needed, builds, adds shell functions):

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh
source ~/.bashrc   # or ~/.zshrc
```

**Manual:**

```bash
git clone https://github.com/Samujalphukan228/jump-cli
cd jump-cli
cargo build --release
mkdir -p ~/.local/bin
cp target/release/jump ~/.local/bin/jump-bin
cat jump.sh >> ~/.bashrc
source ~/.bashrc
```

**Uninstall:**

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh -s -- --uninstall
source ~/.bashrc
```

**Optional preview tools:**

| Tool | Purpose |
|------|---------|
| `chafa` | Better image previews |
| `ffmpeg` | Video thumbnails and audio metadata |

```bash
# Arch
sudo pacman -S chafa ffmpeg
# Debian/Ubuntu
sudo apt install chafa ffmpeg
# macOS
brew install chafa ffmpeg
```

**Terminal font:** A [Nerd Font](https://www.nerdfonts.com/) improves file icons in `exp`. ASCII fallbacks work without it.

---

## jump — directory search

```bash
jump              # open search TUI
jump src          # search for "src"
jump api          # substring match
jump nxb          # initialism — matches nexxupp-backend
jump -            # go back to previous directory
jump "nexx src"   # multi-segment — src inside nexx paths
```

**Pins:**

```bash
jump --pin work              # pin cwd as @work
jump --pin api ~/projects/api
jump work                    # instant jump to pin
jump --unpin work
jump --list                  # history + pins dashboard
```

**Flags:**

| Flag | Default | Description |
|------|---------|-------------|
| `--local-depth` | `4` | Search depth from current directory |
| `--depth` | `6` | Search depth from home/root |
| `--root` | `$HOME` | Override search root |
| `--all` | off | Skip local-first search |
| `--respect-gitignore` | off | Skip gitignored directories |
| `--list` | — | Open history and pins dashboard |
| `--explore` / `-e` | — | Open file explorer |
| `--pin <name> [path]` | — | Pin a directory |
| `--unpin <name>` | — | Remove a pin |

### Jump TUI keys

| Key | Action |
|-----|--------|
| Type | Live search |
| `↑` `↓` / `Ctrl+p` `Ctrl+n` | Move selection |
| `Enter` | Jump to directory |
| `Esc` / `Ctrl+c` | Cancel |
| `Ctrl+u` | Clear input |
| `Ctrl+w` | Delete word |
| `Home` / `Ctrl+a` | Start of input |
| `End` / `Ctrl+e` | End of input |

### Jump mouse

| Action | Result |
|--------|--------|
| Scroll | Move selection |
| Click row | Select |
| Double-click row | Jump |
| Click search bar | Place cursor |

Results are ranked: exact → prefix → contains → fuzzy. Frecency breaks ties within the same rank.

---

## exp — file explorer

```bash
exp              # current directory
exp ~/code       # any path
exp /run/media   # external drives
```

Press `c` in the explorer to `cd` into the selected folder and exit.

### Navigation

| Key | Action |
|-----|--------|
| `j`/`k` or `↑`/`↓` | Move |
| `Enter` / `l` / `→` | Open file or folder |
| `h` / `←` / `Backspace` | Parent directory |
| `..` row | Go up |
| `~` | Home |
| `g` / `Home` | First item |
| `G` / `End` | Last item |

### Files

| Key | Action |
|-----|--------|
| `n` / `N` | New file / folder |
| `r` | Rename |
| `d` / `Delete` | Delete (confirm) |
| `y` | Yank (copy) |
| `p` | Paste |
| `x` | Cut |
| `P` | Paste-move |
| `I` | Import to `~/Downloads` |
| `X` | Toggle executable (Unix) |

### View

| Key | Action |
|-----|--------|
| `v` | Toggle preview panel |
| `/` | Live filter |
| `\` | Clear filter |
| `.` | Toggle hidden files |
| `s` | Cycle sort (name/size/modified/type) |
| `o` | Open with menu |
| `m` / `M` | Jump to drive / list drives |

### Exit

| Key | Action |
|-----|--------|
| `c` | cd here and exit |
| `q` / `Esc` | Quit |

### Explorer mouse

| Action | Result |
|--------|--------|
| Scroll | Move selection |
| Click | Select |
| Double-click | Open file or folder |

---

## How search works

1. **Local first** — searches near your cwd, then home (unless `--all`)
2. **Background** — TUI opens immediately; results stream in
3. **Fuzzy** — initialisms (`nxb` → `nexxupp-backend`) and subsequence matching
4. **Skipped by default** — hidden dirs, `node_modules`, `target/`, build caches, IDE folders

Data is stored in `~/.local/share/jump/data.json`.

---

## Project layout

```
jump-cli/
├── install.sh      # installer + uninstaller
├── jump.sh         # shell wrapper (jump + exp functions)
├── src/
│   ├── main.rs     # CLI entry
│   ├── search/     # directory search
│   ├── explorer/   # file manager TUI
│   └── ui/         # jump TUI + shared chrome
└── LICENSE
```

---

## Requirements

- Linux or macOS
- `git`
- Rust (installed automatically by `install.sh` if missing)
- bash or zsh for shell integration

---

MIT © [Samujalphukan228](https://github.com/Samujalphukan228)