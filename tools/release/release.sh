#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "==> Bnpm Release Process"

# Check if working tree is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "ERROR: Working tree is not clean. Commit or stash changes first."
    exit 1
fi

# Ensure we're on main branch
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" != "main" ]; then
    echo "WARNING: Not on main branch (current: $BRANCH)"
    read -rp "Continue anyway? [y/N] " confirm
    if [ "$confirm" != "y" ]; then
        exit 1
    fi
fi

# Run full CI check
echo "==> Running CI checks..."
pnpm ci:check 2>/dev/null || {
    echo "ERROR: CI checks failed."
    exit 1
}

# Create changeset if changes exist
if [ -d .changeset ]; then
    echo "==> Checking for changesets..."
    PENDING=$(ls .changeset/*.md 2>/dev/null | grep -v README.md || true)
    if [ -z "$PENDING" ]; then
        echo "  No pending changesets found."
        read -rp "Create a new changeset? [y/N] " create
        if [ "$create" = "y" ]; then
            npx changeset
        fi
    else
        echo "  Pending changesets found:"
        for f in $PENDING; do
            echo "    - $(basename "$f")"
        done
        read -rp "Version packages? [y/N] " version
        if [ "$version" = "y" ]; then
            npx changeset version
        fi
    fi
fi

echo ""
echo "==> Release prep complete!"
echo ""
echo "Next steps:"
echo "  1. Review changes:  git diff"
echo "  2. Commit:          git add . && git commit -m 'chore(release): prepare release'"
echo "  3. Push:            git push origin $BRANCH"
echo "  4. Tag:             git tag v$(node -p 'require("./package.json").version')"
echo "  5. Push tags:       git push --tags"
echo ""
echo "CI/CD will handle building, testing, and publishing."
