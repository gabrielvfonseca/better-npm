# Dash Homebrew Tap

Homebrew formula for [Dash](https://github.com/org/dash) — a fast, secure JavaScript package manager.

## Installation

```bash
# Add the tap
brew tap org/dash

# Install the CLI
brew install dash

# Or install the development version from HEAD
brew install --head dash
```

## Usage

```bash
# Initialize a new project
dash init my-project
cd my-project

# Install dependencies
dash install

# Add a package
dash install lodash

# Remove a package
dash remove lodash

# Search for packages
dash search react

# Run a script
dash run build

# View cache stats
dash cache stats
```

## Upgrading

```bash
brew update
brew upgrade dash
```

## Uninstalling

```bash
brew uninstall dash
brew untap org/dash
```

## Configuration

Dash looks for configuration in the following locations:

1. `./.dash.toml` — project-level config
2. `~/.config/dash/config.toml` — user-level config
3. `/usr/local/etc/dash.toml` — system-level config (Homebrew install)

## Directory Layout

When installed via Homebrew:

| Path | Purpose |
|---|---|
| `$(brew --prefix)/bin/dash` | Binary |
| `$(brew --prefix)/etc/dash.toml` | System config |
| `$(brew --prefix)/var/dash/cache` | Package cache |
| `$(brew --prefix)/var/dash/data` | Persistent data |
| `$(brew --prefix)/share/man/man1/dash.1` | Man page |
| `$(brew --prefix)/etc/bash_completion.d/dash.bash` | Bash completions |
| `$(brew --prefix)/share/zsh/site-functions/_dash` | Zsh completions |
| `$(brew --prefix)/share/fish/vendor_completions.d/dash.fish` | Fish completions |

## Development

```bash
# Install from local checkout
brew install --build-from-source Formula/dash.rb

# Test the formula
brew test dash

# Audit the formula
brew audit --new-formula Formula/dash.rb
```

## Release Process

1. Tag a new version: `git tag v0.1.0 && git push --tags`
2. GitHub Actions builds release artifacts and bottles
3. Run `make update` to bump the formula's version and hashes
4. Commit and push the updated formula
5. Users can now `brew upgrade dash`
