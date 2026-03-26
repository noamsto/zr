# Fish completions for zr
# Source arg: complete from zoxide entries
# Destination arg: complete from zoxide entries + filesystem

function __zr_needs_source
    set -l tokens (commandline -opc)
    set -l ntokens (count $tokens)
    # Only the command itself, no positional args yet
    for i in (seq 2 $ntokens)
        switch $tokens[$i]
            case '-*'
                continue
            case '*'
                return 1
        end
    end
    return 0
end

function __zr_needs_dest
    set -l tokens (commandline -opc)
    set -l ntokens (count $tokens)
    set -l positional 0
    for i in (seq 2 $ntokens)
        switch $tokens[$i]
            case '-*'
                continue
            case '*'
                set positional (math $positional + 1)
        end
    end
    test $positional -eq 1
end

function __zr_zoxide_paths
    zoxide query -ls 2>/dev/null | string replace -r '^\s*[\d.]+\s+' ''
end

# Flags
complete -c zr -s n -l dry-run -d 'Preview changes without executing'
complete -c zr -s v -l verbose -d 'Show each zoxide entry being updated'
complete -c zr -l completions -d 'Generate shell completions' -r -f -a "bash elvish fish powershell zsh"
complete -c zr -s h -l help -d 'Print help'

# Source: zoxide entries only
complete -c zr -n __zr_needs_source -f -a '(__zr_zoxide_paths)'

# Destination: zoxide entries + filesystem dirs
complete -c zr -n __zr_needs_dest -F -a '(__zr_zoxide_paths)'
