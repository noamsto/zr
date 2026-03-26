#!/usr/bin/env bash
# Bash completions for zr
# Source arg: complete from zoxide entries
# Destination arg: complete from zoxide entries + filesystem dirs

_zr() {
    local cur prev words cword
    _init_completion || return

    # Handle flags
    case "$cur" in
        -*)
            COMPREPLY=($(compgen -W "-n --dry-run -v --verbose --completions -h --help" -- "$cur"))
            return
            ;;
    esac

    # Count positional args (skip flags)
    local positional=0
    for ((i = 1; i < cword; i++)); do
        case "${words[i]}" in
            -*) continue ;;
            *) ((positional++)) ;;
        esac
    done

    local zoxide_paths
    zoxide_paths=$(zoxide query -ls 2>/dev/null | sed 's/^[[:space:]]*[0-9.]*[[:space:]]*//')

    case $positional in
        0)
            # Source: zoxide entries only
            COMPREPLY=($(compgen -W "$zoxide_paths" -- "$cur"))
            ;;
        1)
            # Destination: zoxide entries + directories
            COMPREPLY=($(compgen -W "$zoxide_paths" -- "$cur"))
            COMPREPLY+=($(compgen -d -- "$cur"))
            ;;
    esac
}

complete -F _zr zr
