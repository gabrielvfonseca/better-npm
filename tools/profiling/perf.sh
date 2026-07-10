#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-help}"
TARGET="${2:-apps/api/dist/index.js}"
PROFILE_DIR="$ROOT_DIR/profiling"
mkdir -p "$PROFILE_DIR"

case "$MODE" in
    cpu)
        echo "==> CPU profiling..."
        echo "  Target: $TARGET"
        echo "  Duration: ${DURATION:-30}s"
        echo ""

        if [ ! -f "$TARGET" ]; then
            echo "ERROR: Target not found: $TARGET"
            echo "Build the project first: bun run build --filter=@dash/api"
            exit 1
        fi

        node --cpu-prof \
             --cpu-prof-dir="$PROFILE_DIR" \
             --cpu-prof-name="cpu.$(date +%s).cpuprofile" \
             "$TARGET" &
        PID=$!
        sleep "${DURATION:-30}"
        kill "$PID" 2>/dev/null || true
        wait "$PID" 2>/dev/null || true

        echo "CPU profile saved to $PROFILE_DIR/"
        ;;

    heap)
        echo "==> Heap profiling..."
        echo "  Target: $TARGET"
        echo "  Duration: ${DURATION:-30}s"
        echo ""

        if [ ! -f "$TARGET" ]; then
            echo "ERROR: Target not found: $TARGET"
            exit 1
        fi

        node --heap-prof \
             --heap-prof-dir="$PROFILE_DIR" \
             --heap-prof-name="heap.$(date +%s).heapprofile" \
             "$TARGET" &
        PID=$!
        sleep "${DURATION:-30}"
        kill "$PID" 2>/dev/null || true
        wait "$PID" 2>/dev/null || true

        echo "Heap profile saved to $PROFILE_DIR/"
        ;;

    flamegraph)
        echo "==> Generating flamegraph..."
        echo "  Target: $TARGET"

        if ! command -v 0x &> /dev/null; then
            echo "  Installing 0x..."
            npm install -g 0x 2>/dev/null || bun install -g 0x 2>/dev/null || {
                echo "ERROR: Could not install 0x. Install manually: npm install -g 0x"
                exit 1
            }
        fi

        if command -v cargo &> /dev/null && command -v cargo-flamegraph &> /dev/null; then
            echo "  Also generating Rust flamegraph..."
            cargo flamegraph --root --output "$PROFILE_DIR/rust-flamegraph.svg" 2>/dev/null || true
        fi

        0x -o "$PROFILE_DIR" "$TARGET"
        echo "Flamegraph generated in $PROFILE_DIR/"
        ;;

    trace)
        echo "==> Tracing API requests..."
        BASE_URL="${DASH_BASE_URL:-http://localhost:3000}"
        echo ""

        endpoints=(
            "$BASE_URL/health"
            "$BASE_URL/v1/packages"
            "$BASE_URL/v1/packages/lodash"
            "$BASE_URL/v1/search?q=react"
        )

        for url in "${endpoints[@]}"; do
            curl -s -o /dev/null -w "  %{http_code} %{time_total}s - %{url_effective}\n" "$url" || echo "  FAILED - $url"
        done
        ;;

    analyze)
        echo "==> Analyzing profiles..."
        echo ""

        # Analyze CPU profiles
        if ls "$PROFILE_DIR"/*.cpuprofile 1>/dev/null 2>&1; then
            for f in "$PROFILE_DIR"/*.cpuprofile; do
                echo "  CPU Profile: $(basename "$f")"
                node -e "
                    const p = require('$(realpath "$f")');
                    if (!p.nodes || !p.codeInfos) {
                        console.log('    (unknown format)');
                        return;
                    }
                    const top = p.nodes
                        .filter(n => n.hitCount > 0)
                        .sort((a, b) => b.hitCount - a.hitCount)
                        .slice(0, 10);
                    top.forEach(n => {
                        const fn = p.codeInfos[n.codeInfoId]?.functionName || '(anonymous)';
                        const file = p.codeInfos[n.codeInfoId]?.url || 'unknown';
                        console.log('    ' + fn + ' (' + file + ') - ' + n.hitCount + ' hits');
                    });
                " 2>/dev/null || echo "    (could not parse profile)"
                echo ""
            done
        else
            echo "  No CPU profiles found in $PROFILE_DIR/"
        fi

        # Analyze heap profiles
        if ls "$PROFILE_DIR"/*.heapprofile 1>/dev/null 2>&1; then
            for f in "$PROFILE_DIR"/*.heapprofile; do
                echo "  Heap Profile: $(basename "$f")"
                node -e "
                    const p = require('$(realpath "$f")');
                    if (p.head) {
                        let total = 0, count = 0;
                        function walk(node) {
                            if (!node) return;
                            total += node.selfSize || 0;
                            count++;
                            if (node.children) node.children.forEach(walk);
                        }
                        walk(p.head);
                        console.log('    Total self size: ' + (total / 1024 / 1024).toFixed(2) + ' MB');
                        console.log('    Nodes: ' + count);
                    }
                " 2>/dev/null || echo "    (could not parse profile)"
                echo ""
            done
        else
            echo "  No heap profiles found in $PROFILE_DIR/"
        fi
        ;;

    cargo)
        echo "==> Rust cargo profiling..."
        if ! command -v cargo &> /dev/null; then
            echo "ERROR: cargo not found."
            exit 1
        fi
        CRATE="${2:-}"
        if [ -z "$CRATE" ]; then
            echo "Usage: $0 cargo <crate-name>"
            echo "Available crates:"
            ls -d crates/*/ 2>/dev/null | sed 's/crates\///;s/\///' | sed 's/^/  /'
            exit 1
        fi
        cargo flamegraph --root -p "$CRATE" --output "$PROFILE_DIR/${CRATE}-flamegraph.svg"
        echo "Flamegraph saved to $PROFILE_DIR/${CRATE}-flamegraph.svg"
        ;;

    help|*)
        echo "Usage: $0 <mode> [target]"
        echo ""
        echo "Modes:"
        echo "  cpu               Run CPU sampling profiler"
        echo "  heap              Run heap profiler"
        echo "  flamegraph        Generate flamegraph with 0x"
        echo "  trace             Quick latency trace of API endpoints"
        echo "  analyze           Analyze existing profiles"
        echo "  cargo <crate>     Generate Rust flamegraph for a crate"
        echo ""
        echo "Environment variables:"
        echo "  DASH_BASE_URL     Base URL for API requests (default: http://localhost:3000)"
        echo "  DURATION          Profiling duration in seconds (default: 30)"
        echo ""
        echo "Examples:"
        echo "  $0 cpu apps/registry/dist/index.js"
        echo "  DURATION=60 $0 heap"
        echo "  $0 trace"
        echo "  $0 analyze"
        echo "  $0 cargo api"
        ;;
esac
