# Contributing to the Dash Homebrew Tap

## Adding New Dependencies

If Dash requires new system-level dependencies:

1. Add the dependency with `depends_on` in `Formula/dash.rb`
2. Add required build dependencies with `depends_on "... => :build"`
3. Run `brew audit --strict Formula/dash.rb` to verify

## Building Bottles Locally

```bash
# Build from source
brew install --build-from-source Formula/dash.rb

# Test the installed binary
brew test dash

# Create a bottle
brew bottle --no-rebuild --root-url https://github.com/org/dash/releases/download/v0.1.0 dash
```

## Releasing a New Version

### Automated (preferred)

1. Push a new tag to the main repo: `git tag v0.1.0 && git push --tags`
2. The [Homebrew workflow](.github/workflows/homebrew.yml) handles everything:
   - Builds bottles for all platforms
   - Uploads them to the GitHub release
   - Updates the `org/homebrew-dash` tap's formula

### Manual

```bash
# 1. Set the version
VERSION="0.1.0"

# 2. Update the formula
make update VERSION=$VERSION

# 3. Test locally
brew install --build-from-source Formula/dash.rb
brew test dash
brew audit --strict Formula/dash.rb

# 4. Commit and push
git add Formula/dash.rb
git commit -m "dash $VERSION"
git push
```

## Formula Structure

```
homebrew/
├── Formula/
│   └── dash.rb          # Main formula (tap)
├── dash.rb              # Standalone formula (brew install from URL)
├── aliases/
│   └── dash-dev         # Alias for development version
├── scripts/
│   └── update-formula.sh # Helper to bump version/sha256
├── .github/workflows/
│   └── homebrew.yml     # CI workflow for bottle builds
├── Makefile             # Common tasks
├── CONTRIBUTING.md      # This file
└── README.md            # User-facing docs
```

## Testing

```bash
# Run the full CI check locally
make ci

# Test with offline mode (no network)
HOMEBREW_TEST_OFFLINE=1 brew test dash
