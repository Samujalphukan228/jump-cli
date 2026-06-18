#!/bin/sh
set -e

REPO="https://github.com/Samujalphukan228/jump-cli"
BIN_NAME="jump-bin"
BIN_DIR="$HOME/.local/bin"

RESET="\033[0m"
BOLD="\033[1m"
DIM="\033[2m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
MAGENTA="\033[35m"
BLUE="\033[34m"
WHITE="\033[37m"
BR_BLACK="\033[90m"
BR_CYAN="\033[96m"
BR_BLUE="\033[94m"
BR_MAGENTA="\033[95m"

rgb() { printf "\033[38;2;%s;%s;%sm" "$1" "$2" "$3"; }
bg_rgb() { printf "\033[48;2;%s;%s;%sm" "$1" "$2" "$3"; }

hide_cursor() { printf "\033[?25l"; }
show_cursor() { printf "\033[?25h"; }
clear_screen() { printf "\033[2J\033[H"; }

TERM_WIDTH=$(tput cols 2>/dev/null || echo 80)

info()    { printf "  ${BOLD}${CYAN}▸${RESET} %s\n" "$1"; }
success() { printf "  ${BOLD}${GREEN}✔${RESET} %s\n" "$1"; }
warn()    { printf "  ${BOLD}${YELLOW}▲${RESET} %s\n" "$1"; }
die()     { show_cursor; printf "  ${BOLD}${RED}✖${RESET} %s\n" "$1" >&2; exit 1; }

SPINNER_PID=""

spinner_start() {
    _label="$1"
    (
        _frames="⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏"
        _colors="${CYAN} ${BR_CYAN} ${BLUE} ${BR_BLUE} ${MAGENTA} ${BR_MAGENTA} ${CYAN} ${BR_CYAN} ${BLUE} ${BR_BLUE}"
        _i=0
        while true; do
            _i=$(( (_i + 1) % 10 ))
            _f=$(printf "%s" "$_frames" | cut -d" " -f$((_i + 1)))
            _c=$(printf "%s" "$_colors" | cut -d" " -f$((_i + 1)))
            printf "\r  ${BOLD}${_c}%s${RESET} ${WHITE}%s${RESET}" "$_f" "$_label"
            sleep 0.08
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

trap 'spinner_stop; show_cursor' EXIT INT TERM

# ── Animated Logo ──────────────────────────────────────────────────────────────

print_logo() {
    clear_screen
    hide_cursor

    _col=$(( (TERM_WIDTH - 52) / 2 ))
    [ $_col -lt 1 ] && _col=1

    # Top sweep
    printf "\033[1;${_col}H"
    _i=0
    while [ $_i -lt 52 ]; do
        printf "$(rgb 0 $(( 150 + _i * 2 )) 255)━${RESET}"
        _i=$((_i + 1))
    done
    sleep 0.1
    printf "\033[1;${_col}H"
    _i=0
    while [ $_i -lt 52 ]; do
        printf "${BR_BLACK}━${RESET}"
        _i=$((_i + 1))
    done
    sleep 0.1

    _lines="
     ██╗██╗   ██╗███╗   ███╗██████╗ 
     ██║██║   ██║████╗ ████║██╔══██╗
     ██║██║   ██║██╔████╔██║██████╔╝
██   ██║██║   ██║██║╚██╔╝██║██╔═══╝ 
╚█████╔╝╚██████╔╝██║ ╚═╝ ██║██║     
 ╚════╝  ╚═════╝ ╚═╝     ╚═╝╚═╝    
"

    _start_row=3
    _logo_col=$(( (TERM_WIDTH - 36) / 2 ))
    [ $_logo_col -lt 1 ] && _logo_col=1

    # Ghost pass
    _r=0
    echo "$_lines" | while IFS= read -r _line; do
        [ -z "$_line" ] && continue
        printf "\033[$((_start_row + _r));${_logo_col}H"
        printf "${BR_BLACK}%s${RESET}" "$_line"
        _r=$((_r + 1))
        sleep 0.04
    done
    sleep 0.15

    # Color reveal
    _r=0
    echo "$_lines" | while IFS= read -r _line; do
        [ -z "$_line" ] && continue
        case $_r in
            0) _c=$(rgb 60 180 255) ;;
            1) _c=$(rgb 80 200 255) ;;
            2) _c=$(rgb 100 220 240) ;;
            3) _c=$(rgb 80 240 200) ;;
            4) _c=$(rgb 60 255 180) ;;
            5) _c=$(rgb 40 255 160) ;;
        esac
        printf "\033[$((_start_row + _r));${_logo_col}H"
        printf "${BOLD}${_c}%s${RESET}" "$_line"
        _r=$((_r + 1))
        sleep 0.07
    done
    sleep 0.2

    # Sparkles
    _s=0
    while [ $_s -lt 12 ]; do
        _sr=$(( (_s * 7 + 3) % 6 ))
        _sc=$(( (_s * 13 + 5) % 30 + 3 ))
        printf "\033[$((_start_row + _sr));$((_logo_col + _sc))H"
        printf "${BOLD}$(rgb 255 255 255)✦${RESET}"
        sleep 0.04
        _s=$((_s + 1))
    done
    sleep 0.1

    # Clean redraw
    _r=0
    echo "$_lines" | while IFS= read -r _line; do
        [ -z "$_line" ] && continue
        case $_r in
            0) _c=$(rgb 60 180 255) ;;
            1) _c=$(rgb 80 200 255) ;;
            2) _c=$(rgb 100 220 240) ;;
            3) _c=$(rgb 80 240 200) ;;
            4) _c=$(rgb 60 255 180) ;;
            5) _c=$(rgb 40 255 160) ;;
        esac
        printf "\033[$((_start_row + _r));${_logo_col}H"
        printf "${BOLD}${_c}%s${RESET}" "$_line"
        _r=$((_r + 1))
    done

    # Badge slide
    _badge_row=$((_start_row + 7))
    _badge_text=" ⚡ jump-cli v0.4.0 "
    _badge_len=20
    _badge_final=$(( (TERM_WIDTH - _badge_len) / 2 ))

    _pos=$((TERM_WIDTH - 5))
    while [ $_pos -gt $_badge_final ]; do
        printf "\033[${_badge_row};${_pos}H"
        printf "$(bg_rgb 20 100 160)${BOLD}${WHITE}%s${RESET}" "$_badge_text"
        printf "\033[${_badge_row};$((_pos + _badge_len))H"
        printf "          "
        sleep 0.02
        _pos=$((_pos - 3))
    done
    printf "\033[${_badge_row};${_badge_final}H"
    printf "$(bg_rgb 20 110 170)${BOLD}${WHITE}%s${RESET}" "$_badge_text"
    sleep 0.15

    # Subtitle fade
    _sub_row=$((_badge_row + 2))
    _sub_text="directory jumper + file explorer"
    _sub_col=$(( (TERM_WIDTH - 31) / 2 ))

    printf "\033[${_sub_row};${_sub_col}H"
    printf "${BR_BLACK}%s${RESET}" "$_sub_text"
    sleep 0.15
    printf "\033[${_sub_row};${_sub_col}H"
    printf "${DIM}${WHITE}%s${RESET}" "$_sub_text"
    sleep 0.15
    printf "\033[${_sub_row};${_sub_col}H"
    printf "${WHITE}%s${RESET}" "$_sub_text"

    # Bottom sweep
    _bot_row=$((_sub_row + 2))
    printf "\033[${_bot_row};${_col}H"
    _i=0
    while [ $_i -lt 52 ]; do
        printf "$(rgb 0 $(( 150 + _i * 2 )) 255)━${RESET}"
        sleep 0.01
        _i=$((_i + 1))
    done
    sleep 0.1
    printf "\033[${_bot_row};${_col}H"
    _i=0
    while [ $_i -lt 52 ]; do
        printf "${BR_BLACK}━${RESET}"
        _i=$((_i + 1))
    done

    printf "\033[$((_bot_row + 2));1H"
    show_cursor
}

# ── Detect Shell ───────────────────────────────────────────────────────────────

detect_shell_rc() {
    case "$SHELL" in
        */zsh)  echo "$HOME/.zshrc" ;;
        */bash) echo "$HOME/.bashrc" ;;
        */fish) echo "$HOME/.config/fish/config.fish" ;;
        *)      echo "$HOME/.bashrc" ;;
    esac
}

# ── Steps ──────────────────────────────────────────────────────────────────────

install_rust_if_needed() {
    if command -v cargo >/dev/null 2>&1; then
        success "Rust already installed ${DIM}($(cargo --version))${RESET}"
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

build() {
    command -v cargo >/dev/null 2>&1 || . "$HOME/.cargo/env"
    command -v git   >/dev/null 2>&1 || die "git not found — please install git first"

    TMP_DIR=$(mktemp -d)

    spinner_start "Cloning jump-cli..."
    git clone --depth 1 "$REPO" "$TMP_DIR/jump-cli" >/dev/null 2>&1
    spinner_stop
    success "Cloned"

    spinner_start "Building jump-cli ${DIM}(first run may take a moment)${RESET}..."
    cd "$TMP_DIR/jump-cli"
    cargo build --release --quiet 2>/dev/null
    spinner_stop
    success "Build complete"

    mkdir -p "$BIN_DIR"
    cp "target/release/jump" "$BIN_DIR/$BIN_NAME"
    chmod +x "$BIN_DIR/$BIN_NAME"
    success "Binary installed ${DIM}→ $BIN_DIR/$BIN_NAME${RESET}"

    cd /
    rm -rf "$TMP_DIR"
}

install_deps() {
    info "Checking optional dependencies..."

    if command -v chafa >/dev/null 2>&1; then
        success "chafa ${DIM}(image preview)${RESET}"
    else
        warn "chafa not found — install for image preview"
        printf "    ${BR_BLACK}arch: ${DIM}sudo pacman -S chafa${RESET}\n"
        printf "    ${BR_BLACK}ubuntu: ${DIM}sudo apt install chafa${RESET}\n"
    fi

    if command -v ffmpeg >/dev/null 2>&1; then
        success "ffmpeg ${DIM}(video/audio preview)${RESET}"
    else
        warn "ffmpeg not found — install for video thumbnails"
        printf "    ${BR_BLACK}arch: ${DIM}sudo pacman -S ffmpeg${RESET}\n"
        printf "    ${BR_BLACK}ubuntu: ${DIM}sudo apt install ffmpeg${RESET}\n"
    fi
}

install_wrapper() {
    RC=$(detect_shell_rc)

    if grep -q "jump-bin" "$RC" 2>/dev/null; then
        warn "Removing old wrapper from $RC"
        cp "$RC" "${RC}.bak"
        sed -i '/^# jump shell wrapper/,/^}/d' "$RC"
        sed -i '/^function jump()/,/^}/d' "$RC"
        sed -i '/^function exp()/,/^}/d' "$RC"
        sed -i '/^# Shortcut: .exp/,/^}/d' "$RC"
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

    success "Wrapper added"
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

# ── Main ───────────────────────────────────────────────────────────────────────

print_logo

printf "\n"

install_rust_if_needed
build
install_deps
install_wrapper
ensure_path

RC=$(detect_shell_rc)

printf "\n"
printf "  ${BOLD}${GREEN}✅ All done!${RESET}\n"
printf "\n"
printf "  Run this once to activate:\n"
printf "\n"
printf "    ${BOLD}$(bg_rgb 40 40 60)${WHITE} source %s ${RESET}\n" "$RC"
printf "\n"
printf "    ${BOLD}${CYAN}jump src${RESET}        ${DIM}search & jump to directory${RESET}\n"
printf "    ${BOLD}${CYAN}jump${RESET}            ${DIM}open search TUI${RESET}\n"
printf "    ${BOLD}${CYAN}exp${RESET}             ${DIM}open file explorer${RESET}\n"
printf "    ${BOLD}${CYAN}jump --list${RESET}     ${DIM}show history & pins${RESET}\n"
printf "    ${BOLD}${CYAN}jump -${RESET}          ${DIM}go back${RESET}\n"
printf "\n"