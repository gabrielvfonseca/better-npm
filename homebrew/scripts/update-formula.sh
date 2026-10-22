#!/usr/bin/env bash
set -euo pipefail

# update-formula.sh — Update the Homebrew formula's version and hashes
# Usage: ./scripts/update-formula.sh <version>

VERSION="${1:-}"
FORMULA_FILE="Formula/bnpm.rb"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.0"
  exit 1
fi

if [ ! -f "$FORMULA_FILE" ]; then
  echo "Error: $FORMULA_FILE not found"
  echo "Run this script from the homebrew/ directory"
  exit 1
fi

echo "Updating bnpm formula to version $VERSION..."
echo ""

# Fetch the source tarball and compute its SHA256
SOURCE_URL="https://github.com/org/bnpm/archive/refs/tags/v${VERSION}.tar.gz"
echo "Fetching source from: $SOURCE_URL"
SOURCE_SHA=$(curl -sL "$SOURCE_URL" | sha256sum | cut -d' ' -f1)
echo "Source SHA256: $SOURCE_SHA"

# Update version in formula
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/version \"[0-9.]*\"/version \"$VERSION\"/" "$FORMULA_FILE"
  sed -i '' "s/sha256 \"[a-f0-9]*\"/sha256 \"$SOURCE_SHA\"/" "$FORMULA_FILE"
else
  sed -i "s/version \"[0-9.]*\"/version \"$VERSION\"/" "$FORMULA_FILE"
  sed -i "s/sha256 \"[a-f0-9]*\"/sha256 \"$SOURCE_SHA\"/" "$FORMULA_FILE"
fi

echo ""
echo "Formula updated successfully!"
echo ""
echo "Changes made to $FORMULA_FILE:"
echo "  - Version: $VERSION"
echo "  - Source SHA256: $SOURCE_SHA"
echo ""
echo "Next steps:"
echo "  1. Review the changes: git diff $FORMULA_FILE"
echo "  2. Test the formula: brew install --build-from-source $FORMULA_FILE"
echo "  3. Run tests: brew test bnpm"
echo "  4. Commit: git add $FORMULA_FILE && git commit -m \"bnpm $VERSION\""
echo "  5. Push: git push"
