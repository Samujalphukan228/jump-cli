# jump shell wrapper — v0.2.0
# Add to ~/.bashrc or ~/.zshrc

function jump() {
    # version check
    if [ "$1" = "--version" ] || [ "$1" = "-v" ]; then
        ~/.local/bin/jump-bin --version
        return 0
    fi

    # go back
    if [ "$1" = "-" ]; then
        local tmp
        tmp=$(mktemp)
        ~/.local/bin/jump-bin - --output "$tmp"
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
        return 0
    fi

    local tmp
    tmp=$(mktemp)

    ~/.local/bin/jump-bin "$@" --output "$tmp"
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