#!/usr/bin/env bash
# Bash completions for zr

_zr() {
    local cur prev words cword
    _init_completion || return

    case "$cur" in
        -*)
            COMPREPLY=($(compgen -W "-n --dry-run -v --verbose --completions -h --help" -- "$cur"))
            return
            ;;
    esac

    local positional=0
    for ((i = 1; i < cword; i++)); do
        case "${words[i]}" in
            -*) continue ;;
            *) ((positional++)) ;;
        esac
    done

    local -a zoxide_paths
    while IFS= read -r line; do
        zoxide_paths+=("$line")
    done < <(zoxide query -ls 2>/dev/null | sed 's/^[[:space:]]*[0-9.]*[[:space:]]*//')

    case $positional in
        0)
            COMPREPLY=($(compgen -W "$(printf '%q\n' "${zoxide_paths[@]}")" -- "$cur"))
            ;;
        1)
            COMPREPLY=($(compgen -W "$(printf '%q\n' "${zoxide_paths[@]}")" -- "$cur"))
            COMPREPLY+=($(compgen -d -- "$cur"))
            ;;
    esac
}

complete -F _zr zr
