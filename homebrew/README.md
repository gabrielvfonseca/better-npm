# Bnpm Homebrew Tap

Homebrew formula for [Bnpm](https://github.com/org/bnpm) — a fast, secure JavaScript package manager.

## Installation

```bash
# Add the tap
brew tap org/bnpm

# Install the CLI
brew install bnpm

# Or install the development version from HEAD
brew install --head bnpm
```

## Usage

```bash
# Initialize a new project
bnpm init my-project
cd my-project

# Install dependencies
bnpm install

# Add a package
bnpm install lodash

# Remove a package
bnpm remove lodash

# Search for packages
bnpm search react

# Run a script
bnpm run build

# View cache stats
bnpm cache stats
```

## Upgrading

```bash
brew update
brew upgrade bnpm
```

## Uninstalling

```bash
brew uninstall bnpm
brew untap org/bnpm
```

## Configuration

Bnpm looks for configuration in the following locations:

1. `./.bnpm.toml` — project-level config
2. `~/.config/bnpm/config.toml` — user-level config
3. `/usr/local/etc/bnpm.toml` — system-level config (Homebrew install)

## Directory Layout

When installed via Homebrew:

| Path | Purpose |
|---|---|
| `$(brew --prefix)/bin/bnpm` | Binary |
| `$(brew --prefix)/etc/bnpm.toml` | System config |
| `$(brew --prefix)/var/bnpm/cache` | Package cache |
| `$(brew --prefix)/var/bnpm/data` | Persistent data |
| `$(brew --prefix)/share/man/man1/bnpm.1` | Man page |
| `$(brew --prefix)/etc/bash_completion.d/bnpm.bash` | Bash completions |
| `$(brew --prefix)/share/zsh/site-functions/_bnpm` | Zsh completions |
| `$(brew --prefix)/share/fish/vendor_completions.d/bnpm.fish` | Fish completions |

## Development

```bash
# Install from local checkout
brew install --build-from-source Formula/bnpm.rb

# Test the formula
brew test bnpm

# Audit the formula
brew audit --new-formula Formula/bnpm.rb
```

## Release Process

1. Tag a new version: `git tag v0.1.0 && git push --tags`
2. GitHub Actions builds release artifacts and bottles
3. Run `make update` to bump the formula's version and hashes
4. Commit and push the updated formula
5. Users can now `brew upgrade bnpm`
