<div align="center">

<img src="https://res.cloudinary.com/dzulab559/image/upload/v1781253195/jump_1_g32ryv.png" width="600" alt="jump-cli" />

<br/>

**cd at the speed of thought.**

<br/>

[![version](https://img.shields.io/badge/version-0.2.0-blue?style=flat-square)](https://github.com/Samujalphukan228/jump-cli/releases)
[![license](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS-lightgrey?style=flat-square)]()
[![built with](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square)](https://www.rust-lang.org)

<br/>

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh
```

</div>

<br/>

---

<br/>

## The problem

You know the folder exists. You know what it's called. But `cd` wants the full path — and you have to remember where it lives, how deep it is, which project it belongs to.

```bash
# what you have to do today
cd /home/sam/work/clients/nexxupp/nexxupp-backend/src/api

# what jump-cli lets you do
jump api
```

No config. No bookmarks. No training period. Works instantly on any machine.

<br/>

---

<br/>

## Install

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master/install.sh | sh
```

The installer handles everything:

- Installs Rust automatically if you don't have it
- Clones, builds, and places the binary at `~/.local/bin/jump-bin`
- Injects the shell wrapper into your `.bashrc` or `.zshrc`
- Adds `~/.local/bin` to your `PATH` if needed

Reload your shell once and you're done:

```bash
source ~/.bashrc   # or ~/.zshrc
```

**Prefer to do it manually?**

```bash
git clone https://github.com/Samujalphukan228/jump-cli
cd jump-cli
sh install.sh
```

<br/>

---

<br/>

## Usage

```bash
jump src                   # jump to the nearest directory named src
jump api                   # jump to anything containing "api"
jump nxb                   # initialism — matches nexxupp-backend
jump logs                  # finds any logs directory across your system
jump -                     # go back to your previous directory
jump "nexxupp src"         # multi-segment — find src inside a nexxupp path
jump src --all             # search everywhere, not just local first
jump src --root /work      # search from a specific root
jump src --local-depth 3   # limit depth from current directory
jump src --depth 4         # limit depth from home
jump src --respect-gitignore  # skip dist/, .next/, venv/, etc.
```

**One match → instant jump. Multiple matches → pick from a list.**

```
jump searching for src

  1 [ exact] ★32  /home/sam/projects/myapp/src
  2 [prefix]       /home/sam/projects/myapp/src-old
  3 [  cont]       /home/sam/projects/my-src-utils
  4 [ fuzzy]       /home/sam/projects/search-core
  5                Search everywhere

Pick (0 or q to cancel): 1
→ /home/sam/projects/myapp/src
```

Every result is tagged — you always know why it matched. Frequently visited directories show a `★` score so your most-used folders naturally rise to the top.

<br/>

---

<br/>

## Pins

Bookmark any folder to a short name and jump there instantly — no search, no picker.

```bash
jump --pin work                              # pin cwd as "work"
jump --pin nxb ~/nexxupp/nexxupp-backend     # pin a specific path
jump work                                    # instant jump to pinned folder
jump --unpin work                            # remove the pin
jump --list                                  # see all pins + jump history
```

Pins are checked before any search, so they always win.

<br/>

---

<br/>

## Jump back

```bash
jump -    # go back to where you were before the last jump
```

Works like `cd -` but across any jump, not just the last `cd`.

<br/>

---

<br/>

## History & frecency

jump-cli silently tracks where you go. Folders you visit often get a `★` score that floats them above cold results within the same match tier — without ever overriding a better name match.

```bash
jump --list    # top 20 dirs by frecency score + all pins
```

```
Top jumped directories:

   1. ★128  /home/sam/nexxupp/nexxupp-backend  (32 visits)
   2. ★64   /home/sam/nexxupp/nexxupp-frontend  (16 visits)
   3. ★8    /home/sam/code/jump-cli  (4 visits)

Pins:

  @nxb → /home/sam/nexxupp/nexxupp-backend
  @work → /home/sam/work
```

Data lives in `~/.local/share/jump/data.json`. Delete it anytime to reset.

<br/>

---

<br/>

## How the search works

#### Local first

jump-cli searches your current directory before scanning home. Inside a large monorepo this is significantly faster — most of the time your target is nearby.

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

1. **Initialism** — splits the directory name on `-` and `_`, collects first letters, checks if your query appears in that string. `nxb` → `nexxupp` + `backend` → initials `nb`. Match.
2. **Subsequence** — every character in your query must appear in order inside the directory name. `nxb` matches `noxbuild` because n→o→x→b→u→i→l→d hits all three.

#### Safe fuzzy jumps

A fuzzy match never auto-jumps. It always prompts — because a fuzzy match is a guess, and guesses need confirmation.

#### Multi-segment queries

Narrow by parent folder when you have the same directory name in many projects:

```bash
jump "nexxupp src"    # finds src only inside paths containing "nexxupp"
jump "backend api"    # finds api only inside paths containing "backend"
```

<br/>

---

<br/>

## What gets skipped

jump-cli never wastes time on:

| Skipped | Reason |
|---------|--------|
| Dotfiles and hidden dirs | `.git`, `.cache`, `.config`, etc. — subtree pruned entirely |
| `node_modules` | Pruned at root — nothing inside is ever walked |
| Rust build dirs | `target/` containing `CACHEDIR.TAG` — a real `target-corp` is kept |
| Gitignored dirs | `dist/`, `.next/`, `venv/`, `__pycache__/` etc. with `--respect-gitignore` |

<br/>

---

<br/>

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--local-depth` | `4` | Search depth from current directory |
| `--depth` | `6` | Search depth from home directory |
| `--root` | `$HOME` | Override the home search root |
| `--all` | off | Skip local-first optimisation, always search everywhere |
| `--respect-gitignore` | off | Prune directories matched by `.gitignore` files |
| `--list` | — | Show top 20 frecency dirs and all pins |
| `--pin <name> [path]` | — | Pin cwd (or a path) to a short name |
| `--unpin <name>` | — | Remove a pin |
| `--output` | — | Write resolved path to a file (used internally by the shell wrapper) |

<br/>

---

<br/>

## vs similar tools

|  | `cd` | `zoxide` | `jump-cli` |
|--|------|----------|------------|
| Zero config | ✅ | ❌ needs training | ✅ |
| Works on a fresh machine | ✅ | ❌ | ✅ |
| Fuzzy name matching | ❌ | ✅ by visit frequency | ✅ by name structure |
| Initialism matching | ❌ | ❌ | ✅ |
| Local-first search | ❌ | ❌ | ✅ |
| No database or index file | ✅ | ❌ | ✅ |
| Ranked results | ❌ | ✅ | ✅ |
| Frecency scoring | ❌ | ✅ | ✅ |
| Pinned bookmarks | ❌ | ❌ | ✅ |
| Multi-segment queries | ❌ | ❌ | ✅ |
| Go back (`jump -`) | ✅ (`cd -`) | ❌ | ✅ |
| Gitignore-aware pruning | ❌ | ❌ | ✅ |

`zoxide` learns from where you've been. jump-cli works from what you know — the name. If you can describe the folder, you can jump to it.

<br/>

---

<br/>

## Requirements

- Linux or macOS
- `git`
- Rust — installed automatically by the installer if missing

<br/>

---

<br/>

<div align="center">

MIT © [Samujalphukan228](https://github.com/Samujalphukan228)

<br/>

*If this saved you time, leave a ⭐ — it helps others find it.*

</div>