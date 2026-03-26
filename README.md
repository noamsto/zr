# zr

Move directories while preserving [zoxide](https://github.com/ajeetdsouza/zoxide) scores.

`zr` reads zoxide's database directly, rewrites matching paths (including all children), and saves it back — preserving both rank and last-accessed time. No shelling out, no score drift.

## Usage

```
zr <source> <destination>
```

```bash
# Move a project directory
zr ~/projects/old-name ~/projects/new-name

# Preview what would change
zr -n ~/projects/old-name ~/projects/new-name

# See each zoxide entry being updated
zr -v ~/projects/old-name ~/projects/new-name
```

### Dry Run Output

```
dry run — no changes will be made

move: /home/user/projects/old-name → /home/user/projects/new-name

zoxide entries to update (3):
  rank:45.0  /home/user/projects/old-name → /home/user/projects/new-name
  rank:12.0  /home/user/projects/old-name/src → /home/user/projects/new-name/src
  rank:8.0   /home/user/projects/old-name/docs → /home/user/projects/new-name/docs
```

## Install

### Nix (flake)

Add to your flake inputs:

```nix
inputs.zr = {
  url = "github:noamsto/zr";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Then add to your packages:

```nix
inputs'.zr.packages.default
```

Shell completions for fish, bash, and zsh are installed automatically.

### Cargo

```
cargo install --git https://github.com/noamsto/zr
```

For completions, run `zr --completions <shell>` and place the output in the appropriate directory for your shell.

## How It Works

1. Validates source exists and destination doesn't
2. Reads zoxide's binary database (`db.zo`) directly
3. Moves the directory (`rename(2)`)
4. Rewrites all matching paths in the database (exact match + children)
5. Atomically saves the updated database (write to tmp, then rename)

Both **rank** and **last_accessed** are preserved exactly — no recalculation, no score inflation from `zoxide add`.

## Shell Completions

Both positional arguments autocomplete from zoxide entries. The destination also completes filesystem directories.

| Shell | Installed to                              |
| ----- | ----------------------------------------- |
| Fish  | `share/fish/vendor_completions.d/zr.fish` |
| Bash  | `share/bash-completion/completions/zr`    |
| Zsh   | `share/zsh/site-functions/_zr`            |

## Development

```bash
nix develop        # enter dev shell
cargo watch -x run # rebuild on changes
nix fmt            # format nix + rust
nix build          # reproducible build
```

## Why Not `zoxide add --score`?

`zoxide add --score` sets the **rank**, but `zoxide query -ls` shows the **frecency** (rank multiplied by a time-decay factor). Doing `remove` + `add` resets `last_accessed` to now, which inflates the displayed score. `zr` avoids this by editing the database directly.

## License

MIT
