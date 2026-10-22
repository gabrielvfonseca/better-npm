# Benchmarks

Performance benchmarking suite for the Bnpm registry.

## Methodology

All benchmarks are run using [hyperfine](https://github.com/sharkdp/hyperfine) with the following settings:

- **Warmup**: 3 runs before measurement
- **Minimum runs**: 10 per benchmark
- **Output**: JSON results in `results/`
- **Environment**: Isolated Docker containers, production build

## Categories

### API Endpoints

| Benchmark | Description |
|-----------|-------------|
| `api-hello-world` | Health check endpoint latency |
| `api-list-packages` | Paginated package listing |
| `api-get-package` | Single package lookup by name |

### Registry Operations

| Benchmark | Description |
|-----------|-------------|
| `registry-publish` | Package publish workflow |
| `registry-resolve` | Version resolution via dist-tags |

### Search

| Benchmark | Description |
|-----------|-------------|
| `search-exact` | Exact name match search |
| `search-fuzzy` | Fuzzy/keyword search |
| `search-suggest` | Autocomplete prefix search |

### Database

| Benchmark | Description |
|-----------|-------------|
| `db-query-packages` | Package table scan with limit |
| `db-query-downloads` | Aggregate download counts |

## Running

```bash
# Full benchmark suite
./tools/benchmarks/run.sh

# Individual benchmark with hyperfine
hyperfine --warmup 3 --min-runs 10 \
  'curl -s -o /dev/null http://localhost:3000/v1/packages'

# Dry run (show commands without executing)
BNPM_BENCH_DRY_RUN=1 ./tools/benchmarks/run.sh
```

## Results

Results are stored as JSON files in `benchmarks/results/` with the format:

```json
{
  "results": [{
    "command": "curl -s -o /dev/null http://localhost:3000/health",
    "mean": 0.002345,
    "stddev": 0.000123,
    "median": 0.002301,
    "min": 0.002100,
    "max": 0.003100,
    "runs": 10
  }]
}
```

## Interpreting Results

- **p50 (median)**: Typical latency under normal conditions
- **p95/p99**: Tail latency — important for SLAs
- **Stddev**: Variability in response times
- **Mean vs Median**: Large gap indicates outliers
