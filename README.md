<div align="center">

<img src="https://res.cloudinary.com/dzulab559/image/upload/v1781250387/PM_1_guibuw.png" width="600" alt="jump-cli" />

<br/>

**cd at the speed of thought.**

<br/>

[![version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/Samujalphukan228/jump-cli/releases)
[![license](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![platform](https://img.shields.io/badge/platform-linux%20%7C%20macOS-lightgrey?style=flat-square)]()
[![built with](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square)](https://www.rust-lang.org)

<br/>

```bash
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/main/install.sh | sh
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
curl -sSf https://raw.githubusercontent.com/Samujalphukan228/jump-cli/main/install.sh | sh
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
jump src             # jump to the nearest directory named src
jump api             # jump to anything containing "api"
jump nxb             # initialism — matches nexxupp-backend
jump logs            # finds any logs directory across your system
jump src --all       # search everywhere, not just local first
jump src --root /work          # search from a specific root
jump src --local-depth 3       # limit depth from current directory
jump src --depth 4             # limit depth from home
```

**One match → instant jump. Multiple matches → pick from a list.**

```
→ searching for src

  1 [ exact]  /home/sam/projects/myapp/src
  2 [prefix]  /home/sam/projects/myapp/src-old
  3 [  cont]  /home/sam/projects/my-src-utils
  4 [ fuzzy]  /home/sam/projects/search-core
  5           Search everywhere

Pick (0 or q to cancel): 1
→ /home/sam/projects/myapp/src
```

Every result is tagged — you always know why it matched.

<br/>

---

<br/>

## How the search works

#### Local first

jump-cli searches your current directory before scanning home. Inside a large monorepo this is significantly faster — most of the time your target is nearby.

#### Ranked results

Results are never returned in arbitrary filesystem order. They're sorted by match quality, best first.

| Rank | Query | Matched name | Reason |
|------|-------|-------------|--------|
| exact | `src` | `src` | name equals query exactly |
| prefix | `src` | `src-old` | name starts with query |
| contains | `src` | `my-src` | query appears anywhere in name |
| fuzzy | `nxb` | `nexxupp-backend` | initialism or subsequence |

#### Fuzzy matching

Two passes, no dependencies, no index file:

1. **Initialism** — splits the directory name on `-` and `_`, collects first letters, checks if your query appears in that string. `nxb` → `nexxupp-backend` → initials `nb` ... wait, `n-e-x-x-u-p-p` + `b-a-c-k-e-n-d` → `nb`. Close enough — the full match is `nexxupp` + `backend` → `nb`.
2. **Subsequence** — every character in your query must appear in order inside the directory name. `nxb` matches `noxbuild` because n → o → x → b → u → i → l → d hits all three.

#### Safe fuzzy jumps

A fuzzy match never auto-jumps. It always prompts — because a fuzzy match is a guess, and guesses need confirmation.

<br/>

---

<br/>

## What gets skipped

jump-cli never wastes time on:

| Skipped | Reason |
|---------|--------|
| Dotfiles and hidden dirs | `.git`, `.cache`, `.config`, etc. — subtree pruned entirely |
| `node_modules` | Pruned at root — nothing inside is ever walked |
| Rust build dirs | `target/` directories containing `CACHEDIR.TAG` only — a real `target-corp` is kept |

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