.PHONY: all install test audit update clean bump lint release

VERSION ?= $(shell git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//')
TAP_DIR ?= $(HOME)/.homebrew-tap

all: lint test

# Install from local source
install:
	brew install --build-from-source --verbose Formula/bnpm.rb

# Install the development (HEAD) version
install-head:
	brew install --HEAD --build-from-source --verbose Formula/bnpm.rb

# Run brew test on the formula
test:
	brew test --verbose bnpm

# Run brew audit
audit:
	brew audit --new-formula Formula/bnpm.rb || true
	brew audit --strict Formula/bnpm.rb

# Update the formula version and hashes
update:
	@test -n "$(VERSION)" || (echo "VERSION is not set"; exit 1)
	@echo "Updating formula to version $(VERSION)..."
	@./scripts/update-formula.sh "$(VERSION)"

# Bump the formula for the version-bump workflow
bump:
	@test -n "$(VERSION)" || (echo "VERSION is not set"; exit 1)
	@echo "Bumping formula sha256 for version $(VERSION)..."
	@curl -sL "https://github.com/org/bnpm/archive/refs/tags/v$(VERSION).tar.gz" | sha256sum | cut -d' ' -f1

# Lint the formula with homebrew-style checks
lint:
	@echo "Checking formula style..."
	@if command -v brew >/dev/null 2>&1; then \
		brew style Formula/bnpm.rb; \
	else \
		echo "Homebrew not installed, skipping style check"; \
	fi
	@echo "Checking Ruby syntax..."
	ruby -c Formula/bnpm.rb

# Clean all build artifacts
clean:
	rm -rf build/
	rm -rf completions/
	rm -rf man/

# Deploy the formula to the homebrew tap
release: lint test
	@test -n "$(VERSION)" || (echo "VERSION is not set"; exit 1)
	@echo "Releasing bnpm $(VERSION)..."
	@echo "Copying formula to tap..."
	@mkdir -p "$(TAP_DIR)"
	@cp Formula/bnpm.rb "$(TAP_DIR)/Formula/bnpm.rb"
	@cd "$(TAP_DIR)" && git add Formula/bnpm.rb && \
		git commit -m "bnpm $(VERSION)" && \
		git push
	@echo "Released bnpm $(VERSION) to $(TAP_DIR)"

# Run all checks before release
ci: lint audit test
	@echo "All checks passed."

# Show formula info
info:
	@echo "Bnpm Homebrew Formula"
	@echo "  Version: $(VERSION)"
	@echo "  Formula: Formula/bnpm.rb"
	@echo "  License: MIT"
	@echo ""
	@echo "Dependencies:"
	@echo "  Build: pkg-config, rust"
	@echo "  Runtime: openssl@3, zstd"
	@echo ""
	@echo "Bottle tags: arm64_sequoia, arm64_sonoma, arm64_ventura, x86_64_linux"
