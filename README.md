# jump-cli

**cd at the speed of thought. explore files without leaving the terminal.**

[![version](https://img.shields.io/badge/version-0.4.0-blue?style=flat-square)](https://github.com/Samujalphukan228/jump-cli/releases)
[![license](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS-lightgrey?style=flat-square)]()
[![built with](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square)](https://www.rust-lang.org)

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh
```

---

## The problem

You know the folder exists. You know what it's called. But `cd` wants the full path — and you have to remember where it lives, how deep it is, which project it belongs to.

```bash
# what you have to do today
cd /home/sam/work/clients/nexxupp/nexxupp-backend/src/api

# what jump-cli lets you do
jump api
```

No config. No bookmarks. No training period. Works instantly on any machine.

Need to browse files, preview images, copy from USB? There's a full file explorer built in too.

```bash
exp # open file explorer in current dir
exp ~/code # open at any path
```

---

## Install

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh
```

The installer handles everything:

- Installs Rust automatically if you don't have it
- Clones, builds, and places the binary at `~/.local/bin/jump-bin`
- Injects the shell wrapper into your `.bashrc` or `.zshrc`
- Adds `~/.local/bin` to your `PATH` if needed
- Checks for optional dependencies (`chafa`, `ffmpeg`)

Reload your shell once and you're done:

```bash
source ~/.bashrc # or ~/.zshrc
```

**Optional dependencies for enhanced previews:**

```bash
# Arch
sudo pacman -S chafa ffmpeg

# Ubuntu / Debian
sudo apt install chafa ffmpeg

# macOS
brew install chafa ffmpeg
```

| Dependency | What it enables |
|-----------|----------------|
| `chafa` | High-quality image preview in terminal |
| `ffmpeg` | Video thumbnails + audio metadata |

Both are optional — everything works without them, just with simpler previews.

**Prefer to do it manually?**

```bash
git clone https://github.com/Samujalphukan228/jump-cli
cd jump-cli
cargo build --release
cp target/release/jump ~/.local/bin/jump-bin
cat jump.sh >> ~/.bashrc
source ~/.bashrc
```

---

## Two tools in one

### ⚡ `jump` — instant directory jumper

```bash
jump src # search & jump to any "src" directory
jump # open interactive search TUI
jump - # go back to previous directory
```

### 📂 `exp` — terminal file explorer

```bash
exp # explore current directory
exp ~/code # explore any path
exp /mnt/usb # explore external drives
```

---

## Jump — directory search

```bash
jump src # jump to the nearest directory named src
jump api # jump to anything containing "api"
jump nxb # initialism — matches nexxupp-backend
jump logs # finds any logs directory across your system
jump - # go back to your previous directory
jump "nexxupp src" # multi-segment — find src inside a nexxupp path
jump src --all # search everywhere, not just local first
jump src --root /work # search from a specific root
jump src --local-depth 3 # limit depth from current directory
jump src --depth 4 # limit depth from home
jump src --respect-gitignore # skip dist/, .next/, venv/, etc.
```

**One match → instant jump. Multiple matches → interactive TUI picker.**

The TUI opens immediately and searches in the background as you type. Results stream in — local matches first, then global.

### Search TUI keybindings

| Key | Action |
|-----|--------|
| Type anything | Live search — results update as you type |
| `↑` `↓` | Navigate results |
| `Ctrl+k` `Ctrl+j` | Navigate results (vim style) |
| `PageUp` `PageDown` | Scroll by page |
| `Enter` | Jump to selected directory |
| `Backspace` | Delete character |
| `Ctrl+u` | Clear search |
| `Ctrl+w` | Delete last word |
| `←` `→` | Move cursor |
| `Home` / `Ctrl+a` | Cursor to start |
| `End` / `Ctrl+e` | Cursor to end |
| `Esc` / `Ctrl+c` | Cancel |

Results are ranked and tagged:

| Badge | Meaning |
|-------|---------|
| `[exact ]` | Name equals query exactly |
| `[prefix]` | Name starts with query |
| `[match ]` | Query appears anywhere in name |
| `[fuzzy ]` | Initialism or subsequence match |
| `★ 32` | Frecency score — higher = more visited |
| `◆` | Local result (found near current dir) |

---

## Explorer — terminal file manager

A full file manager inside your terminal. Browse, create, delete, rename, copy, paste, preview — all without leaving the command line.

```bash
exp # current directory
exp ~/code # specific path
exp ~/Pictures # browse images with preview
```

### Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `PageDown` | Scroll 15 down |
| `PageUp` | Scroll 15 up |
| `g` / `Home` | Jump to first |
| `G` / `End` | Jump to last |
| `Enter` / `l` / `→` | Open folder / open file |
| `h` / `←` / `Backspace` | Go to parent directory |
| `~` | Jump to home |

### File operations

| Key | Action |
|-----|--------|
| `n` | Create new file |
| `N` | Create new folder |
| `r` | Rename selected |
| `d` / `Delete` | Delete (with confirmation) |
| `y` | Yank (copy) selected |
| `p` | Paste yanked item |
| `x` | Cut selected |
| `P` | Paste-move cut item |
| `I` | Import selected to `~/Downloads` |
| `X` | Toggle executable permission (Unix) |

### View & search

| Key | Action |
|-----|--------|
| `v` | Toggle preview panel |
| `/` | Live filter (fuzzy search as you type) |
| `\` | Clear filter |
| `.` | Toggle hidden files |
| `s` | Cycle sort (name → size → modified → type) |
| `o` | Open With menu (auto-detects installed apps) |

### Drives & external devices

| Key | Action |
|-----|--------|
| `m` | Jump to first mounted drive (USB/phone) |
| `M` | Show all mounted drives |

### Exit

| Key | Action |
|-----|--------|
| `c` | **cd here** — exit and jump to selected dir |
| `q` / `Esc` | Quit without changing directory |

---

## Preview panel

Press `v` in the explorer to toggle the preview panel. It shows different content based on file type:

| File type | Preview shows |
|-----------|--------------|
| 📄 Text / Code | Syntax-colored source with line numbers |
| 🖼️ Images | Full color ASCII art + dimensions + megapixels |
| 🎬 Video | Thumbnail frame + duration + codec + FPS + resolution |
| 🎵 Audio | Artist / album / genre / duration + waveform art |
| ⚙️ Binary | Hex dump with ASCII column |
| 📁 Directory | Contents list + file/dir counts + total size |

Preview quality depends on installed tools:

| Tool | Quality |
|------|---------|
| `chafa` installed | Best — high resolution color blocks |
| `viu` installed | Good — half-block characters |
| Neither | Basic — built-in half-block renderer |

Video thumbnails and audio metadata require `ffmpeg`.

---

## Open With

Press `o` on any file to see all installed apps that can open it:

```
📂 Open With

📄 main.rs

▸ 🔧 System Default (xdg-open) 1
💻 VS Code (code) 2
📝 Neovim (nvim) 3
📝 Vim (vim) 4
📝 Nano (nano) 5

⏎ select 1-9 quick pick esc cancel
```

Auto-detects 40+ apps: VS Code, Neovim, Vim, Helix, Sublime, IntelliJ, PyCharm, VLC, MPV, GIMP, Firefox, LibreOffice, and more. Only shows apps actually installed on your system.

---

## USB / Phone / External drives

Plug in a USB drive, phone, or external disk:

```bash
exp # open explorer
# press m → jumps to mounted drive
# press M → shows all drives in status bar
```

**Copy from USB to computer:**

```
m → go to USB
↑↓ → find file
y → yank (copy)
~ → go home
Enter → navigate to destination
p → paste
```

**Quick import to Downloads:**

```
m → go to USB
↑↓ → find file
I → imported to ~/Downloads (one key!)
```

**Move files between devices:**

```
m → go to USB
x → cut file
~ → go home
P → paste-move (works across filesystems)
```

Drives are auto-detected at `/media/*`, `/mnt/*`, and `/run/media/$USER/*`.

---

## Live filter

Press `/` in the explorer to start filtering. Results update as you type with fuzzy matching:

```
🔍 main 3/42

▸ 🦀 main.rs 1.2K 2h
📋 main.py 800B 5d
⚡ main.js 2.1K 1d
```

- **Substring match:** `main` finds `main.rs`, `domain.py`
- **Initials match:** `jc` finds `jump-cli`
- **Fuzzy match:** `mrc` finds `my-rust-crate`
- Matching characters highlighted in yellow
- Counter shows matches vs total: `3/42`
- Navigate with `↑` `↓` while filtering
- `Enter` locks the filter, `Esc` clears it

---

## Pins

Bookmark any folder to a short name and jump there instantly — no search, no picker.

```bash
jump --pin work # pin cwd as "work"
jump --pin nxb ~/nexxupp/nexxupp-backend # pin a specific path
jump work # instant jump to pinned folder
jump --unpin work # remove the pin
jump --list # see all pins + jump history
```

Pins are checked before any search, so they always win.

---

## Jump back

```bash
jump - # go back to where you were before the last jump
```

Works like `cd -` but across any jump, not just the last `cd`.

---

## History & frecency

jump-cli silently tracks where you go. Folders you visit often get a `★` score that floats them above cold results within the same match tier — without ever overriding a better name match.

```bash
jump --list # opens a TUI dashboard showing history + pins
```

Data lives in `~/.local/share/jump/data.json`. Delete it anytime to reset.

---

## How the search works

#### Local first

jump-cli searches your current directory before scanning home. Inside a large monorepo this is significantly faster — most of the time your target is nearby.

#### Background search

The TUI opens immediately. Search runs in a background thread. Local results appear first, then global results stream in — no waiting.

#### Ranked results

Results are sorted by match quality, best first. Frecency breaks ties within the same rank.

| Rank | Query | Matched name | Reason |
|------|-------|-------------|--------|
| exact | `src` | `src` | name equals query exactly |
| prefix | `src` | `src-old` | name starts with query |
| contains | `src` | `my-src` | query appears anywhere in name |
| fuzzy | `nxb` | `nexxupp-backend` | initialism or subsequence |

#### Fuzzy matching

Two passes, no dependencies, no index file:

1. **Initialism** — splits the directory name on `-` and `_`, collects first letters, checks if your query appears in that string. `nxb` → `nexxupp-backend` → initials `nb`. Match.
2. **Subsequence** — every character in your query must appear in order inside the directory name. `nxb` matches `noxbuild` because n→o→x→b→u→i→l→d hits all three.

#### Multi-segment queries

Narrow by parent folder when you have the same directory name in many projects:

```bash
jump "nexxupp src" # finds src only inside paths containing "nexxupp"
jump "backend api" # finds api only inside paths containing "backend"
```

---

## What gets skipped

jump-cli never wastes time on:

| Skipped | Reason |
|---------|--------|
| Hidden dirs | `.git`, `.cache`, `.config`, etc. — subtree pruned entirely |
| `node_modules` | Pruned at root — nothing inside is ever walked |
| Build artifacts | `target/`, `dist/`, `build/`, `__pycache__/`, `.next/` |
| Package caches | `.npm`, `.yarn`, `.cargo`, `.rustup`, `.cache` |
| IDE dirs | `.idea`, `.vscode`, `.vs` |
| VCS dirs | `.git`, `.svn`, `.hg` |
| Rust build dirs | `target/` containing `CACHEDIR.TAG` or `.rustc_info.json` |
| Python venvs | `venv/`, `.venv/`, `env/` with `pyvenv.cfg` |
| Gitignored dirs | Everything in `.gitignore` with `--respect-gitignore` |

---

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--local-depth` | `4` | Search depth from current directory |
| `--depth` | `6` | Search depth from home directory |
| `--root` | `$HOME` | Override the home search root |
| `--all` | off | Skip local-first optimisation, always search everywhere |
| `--respect-gitignore` | off | Prune directories matched by `.gitignore` files |
| `--list` | — | Show frecency dashboard and all pins |
| `--explore` | — | Open file explorer TUI |
| `--pin <name> [path]` | — | Pin cwd (or a path) to a short name |
| `--unpin <name>` | — | Remove a pin |
| `--output` | — | Write resolved path to a file (used internally by the shell wrapper) |

---

## Quick Reference

### Jump (Directory Search)

| Command                        | Description                              |
|--------------------------------|------------------------------------------|
| `jump`                         | Open interactive search TUI              |
| `jump <query>`                 | Search and jump to directory             |
| `jump -`                       | Go back to previous directory            |
| `jump --list`                  | Show history and pins                    |
| `jump --pin <name>`            | Pin current directory                    |
| `jump --unpin <name>`          | Remove a pin                             |

### Explorer (File Manager)

| Command              | Description                        |
|----------------------|------------------------------------|
| `exp`                | Open explorer in current directory |
| `exp <path>`         | Open explorer at specific path     |

### Explorer Keyboard Shortcuts

| Category       | Keys                                      | Action |
|----------------|-------------------------------------------|--------|
| **Navigation** | `j` `k` / `↑` `↓`                         | Move up/down |
|                | `Enter` / `l` / `→`                       | Open file or folder |
|                | `h` / `←` / `Backspace`                   | Go to parent |
|                | `~`                                       | Go to home |
| **File Ops**   | `n` / `N`                                 | New file / folder |
|                | `r`                                       | Rename |
|                | `d` / `Delete`                            | Delete |
|                | `y`                                       | Yank (copy) |
|                | `p`                                       | Paste |
|                | `x` / `P`                                 | Cut / Paste-move |
|                | `I`                                       | Import to `~/Downloads` |
| **View**       | `v`                                       | Toggle preview |
|                | `/`                                       | Live filter |
|                | `.`                                       | Toggle hidden files |
|                | `s`                                       | Cycle sort |
|                | `o`                                       | Open With menu |
| **Drives**     | `m` / `M`                                 | Jump to drive / Show all |
| **Exit**       | `c`                                       | cd here & exit |
|                | `q` / `Esc`                               | Quit |

---

## vs similar tools

| | `cd` | `zoxide` | `ranger` | `jump-cli` |
|--|------|----------|----------|------------|
| Zero config | ✅ | ❌ needs training | ❌ needs config | ✅ |
| Works on a fresh machine | ✅ | ❌ | ✅ | ✅ |
| Fuzzy name matching | ❌ | ✅ by frequency | ❌ | ✅ by name structure |
| Initialism matching | ❌ | ❌ | ❌ | ✅ |
| Local-first search | ❌ | ❌ | ❌ | ✅ |
| Background search | ❌ | ❌ | ❌ | ✅ |
| Ranked results | ❌ | ✅ | ❌ | ✅ |
| Frecency scoring | ❌ | ✅ | ❌ | ✅ |
| Pinned bookmarks | ❌ | ❌ | ✅ | ✅ |
| Multi-segment queries | ❌ | ❌ | ❌ | ✅ |
| Go back (`jump -`) | ✅ (`cd -`) | ❌ | ❌ | ✅ |
| Built-in file explorer | ❌ | ❌ | ✅ | ✅ |
| File preview (images/video) | ❌ | ❌ | ✅ | ✅ |
| Open With menu | ❌ | ❌ | ✅ | ✅ |
| USB/phone support | ❌ | ❌ | ❌ | ✅ |
| Live fuzzy filter | ❌ | ❌ | ❌ | ✅ |
| Copy/paste/move files | ❌ | ❌ | ✅ | ✅ |
| Gitignore-aware pruning | ❌ | ❌ | ❌ | ✅ |

`zoxide` learns from where you've been. `ranger` is a file manager. **jump-cli is both** — a smart directory jumper and a full file explorer in one tool.

---

## Requirements

- Linux or macOS
- `git`
- Rust — installed automatically by the installer if missing

**Optional (for enhanced previews):**

- `chafa` — best quality image preview
- `ffmpeg` — video thumbnails + audio metadata
- `viu` — alternative image preview

---

MIT © [Samujalphukan228](https://github.com/Samujalphukan228)

*If this saved you time, leave a ⭐ — it helps others find it.*