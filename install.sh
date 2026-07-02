#!/bin/sh
# jump-cli installer — builds from source and wires shell functions
set -e

REPO="https://github.com/Samujalphukan228/jump-cli"
BIN_NAME="jump-bin"
BIN_DIR="$HOME/.local/bin"
VERSION="0.4.0"

info()    { printf '  > %s\n' "$1"; }
success() { printf '  ok %s\n' "$1"; }
warn()    { printf '  ! %s\n' "$1"; }
die()     { printf '  error: %s\n' "$1" >&2; exit 1; }

detect_shell_rc() {
    case "$SHELL" in
        */zsh)  printf '%s\n' "$HOME/.zshrc" ;;
        */bash) printf '%s\n' "$HOME/.bashrc" ;;
        */fish) die "fish is not supported yet — use bash or zsh, or load jump.sh manually" ;;
        *)      printf '%s\n' "$HOME/.bashrc" ;;
    esac
}

remove_wrapper() {
    RC=$1
    [ -f "$RC" ] || return 0
    grep -q "jump shell wrapper" "$RC" 2>/dev/null || return 0

    cp "$RC" "${RC}.bak"
    awk '
        /^# jump shell wrapper/ { skip=1; next }
        /^# end jump shell wrapper/ { skip=0; next }
        skip { next }
        { print }
    ' "${RC}.bak" > "$RC"
    success "removed old wrapper from $RC (backup: ${RC}.bak)"
}

uninstall() {
    info "uninstalling jump-cli"
    pkill -f "$BIN_NAME" 2>/dev/null || true

    if [ -f "$BIN_DIR/$BIN_NAME" ]; then
        rm -f "$BIN_DIR/$BIN_NAME"
        success "removed $BIN_DIR/$BIN_NAME"
    fi

    RC=$(detect_shell_rc)
    remove_wrapper "$RC"

    if [ -d "$HOME/.local/share/jump" ]; then
        rm -rf "$HOME/.local/share/jump"
        success "removed ~/.local/share/jump"
    fi

    printf '\n  done. run: source %s\n\n' "$RC"
    exit 0
}

install_rust_if_needed() {
    if command -v cargo >/dev/null 2>&1; then
        success "rust: $(cargo --version)"
        return
    fi
    info "installing rust via rustup"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    # shellcheck disable=SC1091
    . "$HOME/.cargo/env"
    success "rust installed"
}

JUMP_SH=""

build_binary() {
    command -v git >/dev/null 2>&1 || die "git is required"
    command -v cargo >/dev/null 2>&1 || . "$HOME/.cargo/env"

    TMP_DIR=$(mktemp -d)

    info "cloning $REPO"
    git clone --depth 1 "$REPO" "$TMP_DIR/jump-cli" >/dev/null 2>&1

    info "building release binary"
    (cd "$TMP_DIR/jump-cli" && cargo build --release --quiet)

    pkill -f "$BIN_NAME" 2>/dev/null || true
    sleep 0.2
    mkdir -p "$BIN_DIR"
    cp "$TMP_DIR/jump-cli/target/release/jump" "$BIN_DIR/$BIN_NAME"
    chmod +x "$BIN_DIR/$BIN_NAME"
    JUMP_SH="$TMP_DIR/jump-cli/jump.sh"
    [ -f "$JUMP_SH" ] || die "jump.sh missing in repository checkout"
    success "installed $BIN_DIR/$BIN_NAME"

    rm -rf "$TMP_DIR"
}

check_optional_deps() {
    command -v chafa >/dev/null 2>&1 \
        && success "chafa found (better image previews)" \
        || warn "chafa not found — optional, improves image preview"
    command -v ffmpeg >/dev/null 2>&1 \
        && success "ffmpeg found (video/audio previews)" \
        || warn "ffmpeg not found — optional, enables video/audio metadata"
}

install_wrapper() {
    RC=$(detect_shell_rc)
    touch "$RC"
    remove_wrapper "$RC"

    info "installing shell functions into $RC"
    printf '\n' >> "$RC"
    cat "$JUMP_SH" >> "$RC"
    success "added jump and exp to $RC"
}

ensure_path() {
    RC=$(detect_shell_rc)
    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            if ! grep -Fq "$BIN_DIR" "$RC" 2>/dev/null; then
                printf '\nexport PATH="%s:$PATH"\n' "$BIN_DIR" >> "$RC"
                success "added $BIN_DIR to PATH in $RC"
            fi
            ;;
    esac
}

print_header() {
    printf '\n'
    printf '  jump-cli v%s\n' "$VERSION"
    printf '  directory jumper + file explorer\n'
    printf '\n'
}

if [ "$1" = "--uninstall" ] || [ "$1" = "uninstall" ]; then
    uninstall
fi

print_header
install_rust_if_needed
build_binary
check_optional_deps
install_wrapper
ensure_path

RC=$(detect_shell_rc)
printf '\n'
printf '  installed.\n'
printf '  reload your shell:\n'
printf '    source %s\n' "$RC"
printf '\n'
printf '  try:\n'
printf '    jump src\n'
printf '    jump\n'
printf '    exp\n'
printf '\n'
printf '  uninstall: sh install.sh --uninstall\n'
printf '\n'