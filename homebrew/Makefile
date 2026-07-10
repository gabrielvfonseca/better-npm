.PHONY: all install test audit update clean bump lint release

VERSION ?= $(shell git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//')
TAP_DIR ?= $(HOME)/.homebrew-tap

all: lint test

# Install from local source
install:
	brew install --build-from-source --verbose Formula/dash.rb

# Install the development (HEAD) version
install-head:
	brew install --HEAD --build-from-source --verbose Formula/dash.rb

# Run brew test on the formula
test:
	brew test --verbose dash

# Run brew audit
audit:
	brew audit --new-formula Formula/dash.rb || true
	brew audit --strict Formula/dash.rb

# Update the formula version and hashes
update:
	@test -n "$(VERSION)" || (echo "VERSION is not set"; exit 1)
	@echo "Updating formula to version $(VERSION)..."
	@./scripts/update-formula.sh "$(VERSION)"

# Bump the formula for the version-bump workflow
bump:
	@test -n "$(VERSION)" || (echo "VERSION is not set"; exit 1)
	@echo "Bumping formula sha256 for version $(VERSION)..."
	@curl -sL "https://github.com/org/dash/archive/refs/tags/v$(VERSION).tar.gz" | sha256sum | cut -d' ' -f1

# Lint the formula with homebrew-style checks
lint:
	@echo "Checking formula style..."
	@if command -v brew >/dev/null 2>&1; then \
		brew style Formula/dash.rb; \
	else \
		echo "Homebrew not installed, skipping style check"; \
	fi
	@echo "Checking Ruby syntax..."
	ruby -c Formula/dash.rb

# Clean all build artifacts
clean:
	rm -rf build/
	rm -rf completions/
	rm -rf man/

# Deploy the formula to the homebrew tap
release: lint test
	@test -n "$(VERSION)" || (echo "VERSION is not set"; exit 1)
	@echo "Releasing dash $(VERSION)..."
	@echo "Copying formula to tap..."
	@mkdir -p "$(TAP_DIR)"
	@cp Formula/dash.rb "$(TAP_DIR)/Formula/dash.rb"
	@cd "$(TAP_DIR)" && git add Formula/dash.rb && \
		git commit -m "dash $(VERSION)" && \
		git push
	@echo "Released dash $(VERSION) to $(TAP_DIR)"

# Run all checks before release
ci: lint audit test
	@echo "All checks passed."

# Show formula info
info:
	@echo "Dash Homebrew Formula"
	@echo "  Version: $(VERSION)"
	@echo "  Formula: Formula/dash.rb"
	@echo "  License: MIT"
	@echo ""
	@echo "Dependencies:"
	@echo "  Build: pkg-config, rust"
	@echo "  Runtime: openssl@3, zstd"
	@echo ""
	@echo "Bottle tags: arm64_sequoia, arm64_sonoma, arm64_ventura, x86_64_linux"
