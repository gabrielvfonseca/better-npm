#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

OUTPUT_DIR="benchmarks/results"
mkdir -p "$OUTPUT_DIR"

echo "==> Dash Benchmarks"
echo ""

# Check for hyperfine
if ! command -v hyperfine &> /dev/null; then
    echo "WARNING: hyperfine is not installed."
    echo "Install it with: brew install hyperfine (macOS) or cargo install hyperfine"
    echo "Alternatively, set DASH_BENCH_SIMPLE=1 for simple curl timing."
    echo ""
fi

run_benchmark() {
    local name="$1"
    local cmd="$2"
    local output="$OUTPUT_DIR/${name}.json"
    local simple_output="$OUTPUT_DIR/${name}.txt"

    echo "  Running: ${name}..."

    if [ "${DASH_BENCH_DRY_RUN:-0}" = "1" ]; then
        echo "    (dry run) Would execute: $cmd"
        return
    fi

    if command -v hyperfine &> /dev/null && [ "${DASH_BENCH_SIMPLE:-0}" != "1" ]; then
        hyperfine \
            --warmup 3 \
            --min-runs 10 \
            --export-json "$output" \
            --command-name "$name" \
            "$cmd" 2>/dev/null || {
            local exit_code=$?
            echo "    WARNING: Benchmark '${name}' failed (exit code: ${exit_code})"
            return $exit_code
        }

        if [ -f "$output" ]; then
            local mean
            mean=$(jq -r '.results[0].mean // "N/A"' "$output" 2>/dev/null)
            local stddev
            stddev=$(jq -r '.results[0].stddev // "N/A"' "$output" 2>/dev/null)
            echo "    mean: ${mean}s  stddev: ${stddev}s"
        fi
    else
        # Simple timing fallback
        local start end duration
        start=$(date +%s.%N)
        eval "$cmd" > /dev/null 2>&1 || true
        end=$(date +%s.%N)
        duration=$(echo "$end - $start" | bc 2>/dev/null || echo "N/A")
        echo "$duration" > "$simple_output"
        echo "    duration: ${duration}s"
    fi
}

# Run Rust benchmarks
if command -v cargo &> /dev/null; then
    echo "--- Rust Benchmarks ---"
    cargo bench 2>/dev/null || echo "  (cargo bench failed or no benchmarks defined)"
    echo ""
fi

# API benchmarks
echo "--- API Benchmarks ---"
BASE_URL="${DASH_BASE_URL:-http://localhost:3000}"
run_benchmark "api-health" "curl -s -o /dev/null -w '%{http_code}' $BASE_URL/health"
run_benchmark "api-list-packages" "curl -s -o /dev/null $BASE_URL/v1/packages?page=1\&page_size=10"
run_benchmark "api-get-package" "curl -s -o /dev/null $BASE_URL/v1/packages/lodash"

# Registry benchmarks
echo ""
echo "--- Registry Benchmarks ---"
REGISTRY_URL="${DASH_REGISTRY_URL:-http://localhost:4000}"
run_benchmark "registry-resolve" "curl -s -o /dev/null $REGISTRY_URL/v1/resolve?name=lodash\&version=latest"

# Search benchmarks
echo ""
echo "--- Search Benchmarks ---"
run_benchmark "search-exact" "curl -s -o /dev/null '$BASE_URL/v1/search?q=lodash'"
run_benchmark "search-fuzzy" "curl -s -o /dev/null '$BASE_URL/v1/search?q=util'"
run_benchmark "search-suggest" "curl -s -o /dev/null '$BASE_URL/v1/suggest?q=lo'"

# Database benchmarks (requires psql)
if command -v psql &> /dev/null; then
    echo ""
    echo "--- Database Benchmarks ---"
    PG_URL="${DATABASE_URL:-postgresql://dash:dash_dev_password@localhost:5432/dash_dev}"
    run_benchmark "db-query-packages" "psql '$PG_URL' -c 'SELECT * FROM packages LIMIT 100;' -t -q -o /dev/null"
    run_benchmark "db-query-downloads" "psql '$PG_URL' -c 'SELECT package_id, COUNT(*) FROM downloads GROUP BY package_id;' -t -q -o /dev/null"
fi

# Aggregate summary
echo ""
echo "--- Summary ---"
if ls "$OUTPUT_DIR"/*.json 1>/dev/null 2>&1; then
    for f in "$OUTPUT_DIR"/*.json; do
        name=$(basename "$f" .json)
        mean=$(jq -r '.results[0].mean // "N/A"' "$f" 2>/dev/null)
        echo "  ${name}: ${mean}s"
    done
elif ls "$OUTPUT_DIR"/*.txt 1>/dev/null 2>&1; then
    for f in "$OUTPUT_DIR"/*.txt; do
        name=$(basename "$f" .txt)
        duration=$(cat "$f")
        echo "  ${name}: ${duration}s"
    done
fi

echo ""
echo "==> Benchmarks complete. Results in ${OUTPUT_DIR}/"
