# Installation

## From Source

```sh
git clone https://github.com/WyattAu/ldir.git
cd ldir
cargo install --path ldc
```

## System Requirements

- Rust 1.88 or later
- HarfBuzz library (for text shaping)
  - Ubuntu/Debian: `sudo apt-get install libharfbuzz-dev`
  - macOS: `brew install harfbuzz`
  - Windows: HarfBuzz is bundled via `harfbuzz-sys`

## Shell Completions

Shell completions are available in the `completions/` directory:

### Bash

```sh
# System-wide
sudo cp completions/ldc.bash /etc/bash_completion.d/ldc

# User-local
mkdir -p ~/.local/share/bash-completion/completions
cp completions/ldc.bash ~/.local/share/bash-completion/completions/ldc
```

### Zsh

```sh
mkdir -p ~/.zsh/completions
cp completions/_ldc ~/.zsh/completions/
```

### Fish

```sh
cp completions/ldc.fish ~/.config/fish/completions/
```

### PowerShell

```powershell
mkdir -p ~/.local/share/powershell/Completions
cp completions/_ldc.ps1 ~/.local/share/powershell/Completions/
```

## Man Pages

```sh
sudo cp man/ldc.1 /usr/local/share/man/man1/
man ldc
```
