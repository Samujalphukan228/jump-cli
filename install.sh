#!/bin/sh
set -e

REPO="https://github.com/Samujalphukan228/jump-cli"
RAW="https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master"
BIN_NAME="jump-bin"
BIN_DIR="$HOME/.local/bin"

info()    { printf "\033[1;36m==>\033[0m %s\n" "$1"; }
success() { printf "\033[1;32m  ✓\033[0m %s\n" "$1"; }
warn()    { printf "\033[1;33m  !\033[0m %s\n" "$1"; }
die()     { printf "\033[1;31merror:\033[0m %s\n" "$1" >&2; exit 1; }

SPINNER_PID=""

spinner_start() {
    label="$1"
    (
        frames="⣾ ⣽ ⣻ ⢿ ⡿ ⣟ ⣯ ⣷"
        i=0
        n=8
        while true; do
            i=$(( (i + 1) % n ))
            f=$(printf "%s" "$frames" | cut -d" " -f$(( i + 1 )))
            printf "\r  %s %s" "$f" "$label"
            sleep 0.1
        done
    ) &
    SPINNER_PID=$!
}

spinner_stop() {
    if [ -n "$SPINNER_PID" ]; then
        kill "$SPINNER_PID" 2>/dev/null
        wait "$SPINNER_PID" 2>/dev/null || true
        SPINNER_PID=""
        printf "\r\033[2K"
    fi
}

trap 'spinner_stop' EXIT INT TERM

detect_shell_rc() {
    case "$SHELL" in
        */zsh)  echo "$HOME/.zshrc" ;;
        */bash) echo "$HOME/.bashrc" ;;
        */fish) echo "$HOME/.config/fish/config.fish" ;;
        *)      echo "$HOME/.bashrc" ;;
    esac
}

install_rust_if_needed() {
    if command -v cargo >/dev/null 2>&1; then
        success "Rust already installed ($(cargo --version))"
        return
    fi

    info "Rust not found — installing via rustup"
    spinner_start "Downloading rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/_rustup.sh
    spinner_stop
    success "rustup downloaded"

    spinner_start "Installing Rust..."
    sh /tmp/_rustup.sh -s -- -y --no-modify-path >/dev/null 2>&1
    spinner_stop
    success "Rust installed"

    . "$HOME/.cargo/env"
}

install_deps() {
    info "Checking optional dependencies..."

    if command -v chafa >/dev/null 2>&1; then
        success "chafa found (image preview)"
    else
        warn "chafa not found — install for better image preview"
        warn "  arch: sudo pacman -S chafa"
        warn "  ubuntu: sudo apt install chafa"
    fi

    if command -v ffmpeg >/dev/null 2>&1; then
        success "ffmpeg found (video/audio preview)"
    else
        warn "ffmpeg not found — install for video thumbnails"
        warn "  arch: sudo pacman -S ffmpeg"
        warn "  ubuntu: sudo apt install ffmpeg"
    fi
}

build() {
    command -v cargo >/dev/null 2>&1 || . "$HOME/.cargo/env"
    command -v git   >/dev/null 2>&1 || die "git not found — please install git first"

    TMP_DIR=$(mktemp -d)

    spinner_start "Cloning jump-cli..."
    git clone --depth 1 "$REPO" "$TMP_DIR/jump-cli" >/dev/null 2>&1
    spinner_stop
    success "Cloned"

    spinner_start "Building jump-cli (first run may take a moment)..."
    cd "$TMP_DIR/jump-cli"
    cargo build --release --quiet 2>/dev/null
    spinner_stop
    success "Build complete"

    mkdir -p "$BIN_DIR"
    cp "target/release/jump" "$BIN_DIR/$BIN_NAME"
    chmod +x "$BIN_DIR/$BIN_NAME"
    success "Binary installed → $BIN_DIR/$BIN_NAME"

    cd /
    rm -rf "$TMP_DIR"
}

install_wrapper() {
    RC=$(detect_shell_rc)

    # Remove old wrapper if present
    if grep -q "jump-bin" "$RC" 2>/dev/null; then
        warn "Removing old wrapper from $RC"
        # Create backup
        cp "$RC" "${RC}.bak"
        # Remove old jump and exp functions
        sed -i '/^# jump shell wrapper/,/^}/d' "$RC"
        sed -i '/^function jump()/,/^}/d' "$RC"
        sed -i '/^function exp()/,/^}/d' "$RC"
        sed -i '/^# Shortcut: .exp/,/^}/d' "$RC"
        # Clean up empty lines
        sed -i '/^$/N;/^\n$/d' "$RC"
    fi

    info "Installing shell wrapper into $RC"

    cat >> "$RC" << 'WRAPPER'

# jump shell wrapper — v0.4.0
# Directory jumper + file explorer

function jump() {
    if [ "$1" = "-" ]; then
        local tmp=$(mktemp)
        ~/.local/bin/jump-bin - --output "$tmp"
        local exit_code=$?
        if [ $exit_code -ne 0 ]; then rm -f "$tmp"; return 1; fi
        if [ -s "$tmp" ]; then
            local target=$(cat "$tmp"); rm -f "$tmp"; cd "$target" || return 1
        else rm -f "$tmp"; fi
        return 0
    fi

    local tmp=$(mktemp)
    ~/.local/bin/jump-bin "$@" --output "$tmp"
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then rm -f "$tmp"; return 1; fi
    if [ -s "$tmp" ]; then
        local target=$(cat "$tmp"); rm -f "$tmp"; cd "$target" || return 1
    else rm -f "$tmp"; fi
}

# File explorer shortcut
function exp() {
    local tmp=$(mktemp)
    ~/.local/bin/jump-bin --explore "${1:-.}" --output "$tmp"
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then rm -f "$tmp"; return 1; fi
    if [ -s "$tmp" ]; then
        local target=$(cat "$tmp"); rm -f "$tmp"; cd "$target" || return 1
    else rm -f "$tmp"; fi
}
WRAPPER

    success "Wrapper added to $RC"
}

ensure_path() {
    RC=$(detect_shell_rc)

    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            if ! grep -q "$BIN_DIR" "$RC" 2>/dev/null; then
                printf '\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$RC"
                success "$BIN_DIR added to PATH"
            fi
            ;;
    esac
}

reload_shell() {
    RC=$(detect_shell_rc)
    info "Reloading shell config"
    . "$RC" 2>/dev/null || true
    success "Shell config reloaded"
}

# ── main ───────────────────────────────────────────────────────────────────────

printf "\n"
printf "\033[1;36m     ⚡ jump-cli v0.4.0 installer\033[0m\n"
printf "\033[0;90m     directory jumper + file explorer\033[0m\n"
printf "\n"

install_rust_if_needed
build
install_deps
install_wrapper
ensure_path
reload_shell

RC=$(detect_shell_rc)

printf "\n"
printf "\033[1;32m  ✅ All done!\033[0m\n"
printf "\n"
printf "  Run this once to activate:\n"
printf "\n"
printf "    \033[1msource %s\033[0m\n" "$RC"
printf "\n"
printf "  Then try:\n"
printf "\n"
printf "    \033[1;36mjump src\033[0m        search & jump to directory\n"
printf "    \033[1;36mjump\033[0m            open search TUI\n"
printf "    \033[1;36mexp\033[0m             open file explorer\n"
printf "    \033[1;36mjump --list\033[0m     show history & pins\n"
printf "    \033[1;36mjump -\033[0m          go back\n"
printf "\n"