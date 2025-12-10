# changeDir

An intelligent directory bookmarking and navigation tool written in Rust. Works across all shells by outputting directory paths that can be integrated via shell functions.

## Features

- **Cross-shell compatible**: Works with bash, zsh, fish, and any other shell
- **Directory bookmarks**: Save up to 36 frequently used directories
- **Directory history**: Track recently visited directories
- **Smart navigation**: Quick access to bookmarks, subdirectories, and parent directories
- **Colorful output**: Enhanced visual feedback with colored terminal output

## Building

```bash
cargo build --release
```

The binary will be in `target/release/changedir`.

## Installation

Copy the binary to a location in your PATH:

```bash
cp target/release/changedir ~/.local/bin/changedir
# or
sudo cp target/release/changedir /usr/local/bin/changedir
```

## Shell Integration

Since a child process cannot change the parent shell's directory, you need to create a shell function wrapper. Add this to your shell configuration file (`~/.bashrc`, `~/.zshrc`, etc.):

### Bash/Zsh

```bash
cdir() {
    # Check if this is a listing/informational command that shouldn't change directory
    # These commands output information to stdout, not directory paths
    for arg in "$@"; do
        case "$arg" in
            -l|--list|-f|--forget|-F|--forget-all|--bookmark|-?|-h|--help)
                # These commands just print information, don't try to cd
                changedir "$@"
                return
                ;;
        esac
    done
    
    # For navigation commands, capture output and cd to it
    local dir=$(changedir "$@")
    if [ -n "$dir" ] && [ -d "$dir" ]; then
        cd "$dir"
    fi
}
```

### Fish

```fish
function cdir
    # Check if this is a listing/informational command
    # These commands output information to stdout, not directory paths
    for arg in $argv
        if contains -- $arg -l --list -f --forget -F --forget-all --bookmark -? -h --help
            # These commands just print information, don't try to cd
            changedir $argv
            return
        end
    end
    
    # For navigation commands, capture output and cd to it
    set dir (changedir $argv)
    if test -n "$dir" -a -d "$dir"
        cd "$dir"
    end
end
```

After adding the function, reload your shell configuration:
```bash
source ~/.bashrc  # or ~/.zshrc
```

## Usage

## Alias 
The author uses this with a number of single or dual character aliases. Of course you can use any character you like, or have no aliases at all. The use of "z" was 
because it is convenient, at the lefthand side of the bottom row of the keyboard. 

```
    alias z='cdir -c '
    alias zl='cdir -l'
    alias zb='cdir --bookmark'
    alias zf='cdir -f'
    alias zF='cdir -F'
    alias zh='cdir -h'
    alias zH='cdir --help '
    alias back='cdir -back'
```

### List bookmarked directories
```bash
cdir -l
# or
changedir -l
```

### Bookmark current directory
```bash
cdir --bookmark
# or
changedir --bookmark
```

### Forget current directory (if bookmarked)
```bash
cdir -f
```

### Forget all bookmarks
```bash
cdir -F
```

### Choose directory interactively (lists with prefix letters)
```bash
cdir -c
```
This will display bookmarks with prefix letters [0-9, a-z] and prompt for selection.

### Choose directory by letter
```bash
cdir -c a
```
Changes directly to the directory labeled with 'a'.

### Change to previous directory
```bash
cdir -b
```
Changes to the most recently visited directory (from history).

### Change up one directory level
```bash
cdir -u
```
Changes to the parent directory.

### List and select subdirectory
```bash
cdir -d
```
Lists all subdirectories of the current directory with prefix letters [0-9, a-z] and prompts for selection.

### Change to directory by name
```bash
cdir myproject
```
Searches for a directory named "myproject" in:
1. Bookmarked directories
2. Subdirectories of current directory
3. Parent directories (up to 5 levels up)

### Print current directory
```bash
cdir
```
With no arguments, prints the current directory path.

### Verbose/Debug mode
```bash
cdir -v
cdir --verbose
```
Enable verbose output to see what the application is doing. Debug messages are printed to stderr and show:
- File operations (loading/saving bookmarks and history)
- Directory searches and paths being checked
- User input parsing
- Selection indices and directory paths
- Error conditions

Example:
```bash
cdir -v -l
# Shows debug output while listing bookmarks
```

## Data Storage

- **Bookmarks**: Stored in `~/.local/changeDirectory`
- **History**: Stored in `~/.local/changeDirectoryHistory` (last 10 directories)

Both files are plain text with one directory path per line.

## Limitations

- Maximum of 36 bookmarks (to fit within [0-9, a-z] prefix range)
- Directory history limited to last 10 entries
- Directory search in parent directories limited to 5 levels up
