# Contributing to the Bnpm Homebrew Tap

## Adding New Dependencies

If Bnpm requires new system-level dependencies:

1. Add the dependency with `depends_on` in `Formula/bnpm.rb`
2. Add required build dependencies with `depends_on "... => :build"`
3. Run `brew audit --strict Formula/bnpm.rb` to verify

## Building Bottles Locally

```bash
# Build from source
brew install --build-from-source Formula/bnpm.rb

# Test the installed binary
brew test bnpm

# Create a bottle
brew bottle --no-rebuild --root-url https://github.com/org/bnpm/releases/download/v0.1.0 bnpm
```

## Releasing a New Version

### Automated (preferred)

1. Push a new tag to the main repo: `git tag v0.1.0 && git push --tags`
2. The [Homebrew workflow](.github/workflows/homebrew.yml) handles everything:
   - Builds bottles for all platforms
   - Uploads them to the GitHub release
   - Updates the `org/homebrew-bnpm` tap's formula

### Manual

```bash
# 1. Set the version
VERSION="0.1.0"

# 2. Update the formula
make update VERSION=$VERSION

# 3. Test locally
brew install --build-from-source Formula/bnpm.rb
brew test bnpm
brew audit --strict Formula/bnpm.rb

# 4. Commit and push
git add Formula/bnpm.rb
git commit -m "bnpm $VERSION"
git push
```

## Formula Structure

```
homebrew/
├── Formula/
│   └── bnpm.rb          # Main formula (tap)
├── bnpm.rb              # Standalone formula (brew install from URL)
├── aliases/
│   └── bnpm-dev         # Alias for development version
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
HOMEBREW_TEST_OFFLINE=1 brew test bnpm
