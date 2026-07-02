# jump shell wrapper — v0.4.0
# Directory jumper + file explorer
# Installed into ~/.bashrc or ~/.zshrc by install.sh

JUMP_BIN="${JUMP_BIN:-$HOME/.local/bin/jump-bin}"

_jump_cd() {
    local tmp
    tmp=$(mktemp)
    "$JUMP_BIN" "$@" --output "$tmp"
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        rm -f "$tmp"
        return 1
    fi
    if [ -s "$tmp" ]; then
        local target
        target=$(cat "$tmp")
        rm -f "$tmp"
        cd "$target" || return 1
    else
        rm -f "$tmp"
    fi
}

function jump() {
    if [ "$1" = "-" ]; then
        _jump_cd -
        return $?
    fi
    _jump_cd "$@"
}

function exp() {
    _jump_cd --explore "${1:-.}"
}

# end jump shell wrapper