# Installation

## Prerequisites

- Rust toolchain (1.70+): https://rustup.rs
- macOS: Homebrew packages for SQLite and libiconv
  ```bash
  brew install sqlite libiconv
  ```

## Install from Source

```bash
# Clone the repository
git clone https://github.com/your-username/rivulet.git
cd rivulet

# Install to ~/.cargo/bin/
cargo install --path .
```

## Verify Installation

```bash
rivulet --help
```

## Uninstall

```bash
cargo uninstall rivulet
```

## Data Location

The SQLite database is stored at:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/rivulet/rivulet.db` |
| Linux | `~/.local/share/rivulet/rivulet.db` |
| Windows | `C:\Users\<user>\AppData\Roaming\rivulet\rivulet.db` |

To reset all data, delete the `rivulet` directory at the above location.
