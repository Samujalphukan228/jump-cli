# jump.sh
# jump shell wrapper — v0.4.0
# Directory jumper + file explorer
# Add to ~/.bashrc or ~/.zshrc

function jump() {
    # go back to previous directory
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

    # all other commands
    local tmp=$(mktemp)
    ~/.local/bin/jump-bin "$@" --output "$tmp"
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then rm -f "$tmp"; return 1; fi
    if [ -s "$tmp" ]; then
        local target=$(cat "$tmp"); rm -f "$tmp"; cd "$target" || return 1
    else rm -f "$tmp"; fi
}

# File explorer — opens in current dir or specified path
function exp() {
    local tmp=$(mktemp)
    ~/.local/bin/jump-bin --explore "${1:-.}" --output "$tmp"
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then rm -f "$tmp"; return 1; fi
    if [ -s "$tmp" ]; then
        local target=$(cat "$tmp"); rm -f "$tmp"; cd "$target" || return 1
    else rm -f "$tmp"; fi
}