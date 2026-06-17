#!/bin/sh
set -e

REPO="https://github.com/Samujalphukan228/jump-cli"
RAW="https://raw.githubusercontent.com/Samujalphukan228/jump-cli/master"
BIN_NAME="jump-bin"
BIN_DIR="$HOME/.local/bin"

# ── helpers ────────────────────────────────────────────────────────────────────

info()    { printf "\033[1;36m==>\033[0m %s\n" "$1"; }
success() { printf "\033[1;32m  ✓\033[0m %s\n" "$1"; }
warn()    { printf "\033[1;33m  !\033[0m %s\n" "$1"; }
die()     { printf "\033[1;31merror:\033[0m %s\n" "$1" >&2; exit 1; }

# ── spinner ────────────────────────────────────────────────────────────────────

SPINNER_PID=""

spinner_start() {
    label="$1"
    (
        frames="🕐 🕑 🕒 🕓 🕔 🕕 🕖 🕗 🕘 🕙 🕚 🕛"
        i=0
        n=12
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

# ── detect shell rc ────────────────────────────────────────────────────────────

detect_shell_rc() {
    case "$SHELL" in
        */zsh)  echo "$HOME/.zshrc" ;;
        */bash) echo "$HOME/.bashrc" ;;
        *)      echo "$HOME/.bashrc" ;;
    esac
}

# ── step 1: install rust if missing ───────────────────────────────────────────

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

# ── step 2: build ──────────────────────────────────────────────────────────────

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

# ── step 3: install shell wrapper ─────────────────────────────────────────────

install_wrapper() {
    RC=$(detect_shell_rc)

    if grep -q "jump-bin" "$RC" 2>/dev/null; then
        warn "Wrapper already present in $RC — skipping"
        return
    fi

    info "Installing shell wrapper into $RC"
    curl -sSf "$RAW/jump.sh" >> "$RC"
    printf "\n" >> "$RC"
    success "Wrapper added to $RC"
}

# ── step 4: ensure ~/.local/bin is in PATH ────────────────────────────────────

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

# ── step 5: reload shell ──────────────────────────────────────────────────────

reload_shell() {
    RC=$(detect_shell_rc)
    info "Reloading shell config"
    . "$RC" 2>/dev/null || true
    success "Shell config reloaded"
}

# ── main ───────────────────────────────────────────────────────────────────────

printf "\n\033[1mjump-cli installer\033[0m\n\n"

install_rust_if_needed
build
install_wrapper
ensure_path
reload_shell

RC=$(detect_shell_rc)

printf "\n\033[1;32mAll done!\033[0m\n\n"
printf "Run this once to activate in your current terminal:\n\n"
printf "  \033[1msource %s\033[0m\n\n" "$RC"
printf "Then try:\n\n"
printf "  \033[1mjump src\033[0m\n\n"