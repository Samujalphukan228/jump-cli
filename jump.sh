# Jump shell wrapper
# Add to ~/.bashrc or ~/.zshrc

function jump() {
    local tmp
    tmp=$(mktemp)

    # Run binary directly — no subshell capturing stdout
    # Binary writes path to temp file, all prompts go to stderr (your terminal)
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