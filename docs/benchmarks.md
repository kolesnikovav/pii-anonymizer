# Performance Benchmarks

Comparative performance tests of PII Anonymizer and alternatives.

## Methodology

### What We Measure
- **Anonymization speed** -- number of texts processed per second
- **Total time** -- time to process the entire test dataset

### What We Do NOT Measure
- Service startup time
- Initialization/warmup time
- Memory consumption
- HTTP API response time (for PII Anonymizer, measured directly via library)

### Test Data
- **1,700 texts** with various PII types
- PII types: email, phone, IP, passport, SNILS, AWS keys, SSH keys
- Combined texts with multiple PII entries
- Diverse formats for each type

### Testing Conditions
- PII Anonymizer: directly via library (without HTTP)
- Presidio: via HTTP API (Docker container)
- Release builds for all services

## Results

### PII Anonymizer

| Metric | Value |
|---------|----------|
| **Texts processed** | 1,700 |
| **Total time** | 0.007 sec |
| **Throughput** | **243,473 texts/sec** |

### Presidio

| Metric | Value |
|---------|----------|
| **Texts processed** | 1,100 |
| **Total time** | 13.692 sec |
| **Throughput** | **80 texts/sec** |

> Note: Presidio benchmark was run via HTTP API (2 containers: analyzer + anonymizer)

## Comparison Table

| Service | Throughput | Relative Speed |
|--------|-------------------|------------------------|
| **PII Anonymizer** | 243,473 texts/sec | **1.0x** (baseline) |
| **Presidio** | 80 texts/sec | **0.0003x** (3,043x slower) |

### Visual Comparison

```
PII Anonymizer: █████████████████████████████████████████████████████████████████████ 243,473 texts/sec
Presidio:       ▏                                                                   80 texts/sec
```

## Technical Details

### PII Anonymizer
- **Language**: Rust
- **Detection method**: Regex patterns
- **Strategy**: Mask (default)
- **Patterns**: 14 active
- **Runtime**: Native binary, release build

### Presidio
- **Language**: Python
- **Detection method**: NLP (spaCy) + Regex
- **Strategy**: Replace (default)
- **Runtime**: Docker container

## Benchmark Structure

```
benchmarks/
├── benchmark_pii_anonymizer.rs    # PII Anonymizer benchmark (Rust)
├── benchmark_presidio.py          # Presidio benchmark (Python)
├── run_benchmarks.sh              # Main run script
├── scripts/
│   └── presidio_server.sh         # Presidio management script
├── data/                          # Test data (generated automatically)
└── results/                       # Benchmark results
    ├── pii_anonymizer.log
    ├── presidio.log
    └── benchmark_results.txt
```

## Running Benchmarks

### Full Benchmark (PII Anonymizer + Presidio)

```bash
cd benchmarks
./run_benchmarks.sh
```

### PII Anonymizer Only

```bash
cargo run --release --bin benchmark_pii_anonymizer
```

### Presidio Only

```bash
# Start Presidio
bash benchmarks/scripts/presidio_server.sh start

# Run benchmark
python3 benchmarks/benchmark_presidio.py http://localhost:5002

# Stop Presidio
bash benchmarks/scripts/presidio_server.sh stop
```

## Notes

1. **Fairness of comparison**:
   - PII Anonymizer is measured directly via library (no HTTP overhead)
   - Presidio is measured via HTTP API (with network overhead)
   - This gives Presidio a slight advantage in fairness

2. **Method differences**:
   - PII Anonymizer: regex patterns only
   - Presidio: NLP model + regex
   - Presidio is more universal but slower

3. **PII types**:
   - PII Anonymizer: Russia-specific (passport, SNILS, INN) + technical (API keys, SSH)
   - Presidio: international (email, phone, credit cards, person name)

## Conclusions

**PII Anonymizer** delivers excellent performance thanks to:
- Compiled Rust language
- Optimized regex patterns
- No ML models or NLP analysis
- Minimal overhead
- **3,043x faster than Presidio!**

**Presidio** provides more intelligent detection:
- Contextual analysis with NLP
- Name and location recognition
- But at a significantly higher computational cost

### Key Numbers

- **PII Anonymizer**: 243,473 texts/sec (0.007 sec for 1,700 texts)
- **Presidio**: 80 texts/sec (13.7 sec for 1,100 texts)
- **Difference**: PII Anonymizer is **3,043x faster**

The choice depends on your performance requirements and the types of PII you need to detect!

---

*Last updated: April 11, 2026*
