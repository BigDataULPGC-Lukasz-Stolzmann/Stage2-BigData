# Testing & Benchmarking Guide

This document describes the comprehensive testing and benchmarking setup for the Big Data Search Engine SOA project.

## 📁 Test Structure (Separated from Production Code)

```
BigDataProject/
├── services/                        # All microservices (production code)
│   ├── ingestion-service/
│   │   ├── src/                     # Production code
│   │   ├── tests/                   # Service integration tests
│   │   │   └── integration_tests.rs
│   │   └── benches/                 # Service benchmarks
│   │       └── ingestion_benchmark.rs
│   ├── indexing-service/
│   │   ├── src/                     # Production code
│   │   ├── tests/                   # Service integration tests
│   │   │   └── integration_tests.rs
│   │   └── benches/                 # Service benchmarks
│   │       └── indexing_benchmark.rs
│   ├── search-service/
│   │   ├── src/                     # Production code
│   │   ├── tests/                   # Service integration tests
│   │   │   └── integration_tests.rs
│   │   └── benches/                 # Service benchmarks
│   │       └── search_benchmark.rs
│   ├── control-module/              # Orchestration service
│   └── docker-compose.yml           # Service configuration
├── tests/                           # System-wide integration tests
│   └── system_integration_tests.rs
├── scripts/                         # Test and benchmark runners
│   ├── run_tests.sh
│   ├── run_benchmarks.sh
│   └── generate_html_reports.sh
└── benchmark_results/               # Performance analysis reports
    └── html_reports/
```

## 🧪 Testing Levels

### 1. Unit Tests
- **Location**: `{service}/src/` (inline with `#[cfg(test)]`)
- **Purpose**: Test individual functions and modules in isolation
- **Run**: `cargo test --lib` in each service directory

### 2. Integration Tests
- **Location**: `{service}/tests/integration_tests.rs`
- **Purpose**: Test API endpoints and service interactions
- **Requirements**: Service must be running
- **Run**: `cargo test --test integration_tests` in each service directory

### 3. System Integration Tests
- **Location**: `tests/system_integration_tests.rs`
- **Purpose**: Test complete end-to-end workflows
- **Requirements**: All services must be running
- **Run**: `cargo test --test system_integration_tests` from root

## 📊 Benchmarking Levels

### 1. Service-Level Benchmarks (Criterion with HTML Reports)
- **Location**: `{service}/benches/{service}_benchmark.rs`
- **Purpose**: Micro-benchmarks for core functions (tokenization, metadata extraction, etc.)
- **Output**: Beautiful HTML reports with interactive charts
- **Run**: `cargo bench` in each service directory
- **Reports**: `target/criterion/{benchmark}/report/index.html`

### 2. Container Performance Tests
- **Purpose**: Real-world API endpoint response times under load
- **Requirements**: Services running in containers
- **Output**: HTML tables with response time metrics

### 3. System-Wide Workflow Benchmarks
- **Purpose**: End-to-end performance measuring complete book processing pipeline
- **Output**: Comprehensive workflow timing analysis

## 🚀 Quick Start

### Running All Tests
```bash
# Make scripts executable
chmod +x scripts/run_tests.sh scripts/run_benchmarks.sh

# Start services first
docker-compose up --build

# Run all tests
./scripts/run_tests.sh

# View results
open test_results/test_report.html
```

### Running All Benchmarks
```bash
# Start services first
docker-compose up --build

# Run all benchmarks
./scripts/run_benchmarks.sh

# View results
open benchmark_results/html_reports/index.html
```

## 📈 Benchmark Reports

Criterion generates professional HTML reports with:
- **Interactive Charts**: Performance trends, violin plots, regression analysis
- **Statistical Analysis**: Mean, median, standard deviation, outlier detection
- **Performance Comparison**: Between different benchmark runs
- **Trend Detection**: Performance regression detection over time

### Example Benchmarks

#### Ingestion Service
- Header/body text splitting performance
- Large file processing benchmarks

#### Indexing Service
- Text tokenization performance (small and large texts)
- Metadata extraction speed
- Full book processing pipeline

#### Search Service
- Query processing performance
- Result filtering and ranking
- Concurrent search handling

## 🐳 Container Testing

### Prerequisites
```bash
# Start all services
cd services
docker-compose up --build

# Verify services are running
curl http://0.0.0.0:7001/status  # Ingestion
curl http://0.0.0.0:7002/status  # Indexing
curl http://0.0.0.0:7003/status  # Search
```

### Individual Service Tests
```bash
# Test ingestion service
cd services/ingestion-service
cargo test --test integration_tests

# Test indexing service
cd services/indexing-service
cargo test --test integration_tests

# Test search service
cd services/search-service
cargo test --test integration_tests
```

## 🌐 System Integration Tests

Tests the complete workflow:
1. **Workflow Test**: Ingest → Index → Search
2. **Multi-book Test**: Process multiple books concurrently
3. **Resilience Test**: Error handling and edge cases
4. **Concurrency Test**: Concurrent operations across services
5. **Performance Test**: End-to-end timing analysis

## 📋 Test Reports

### HTML Test Report Features
- ✅ Pass/fail status for each test suite
- 📋 Detailed logs for failed tests
- 🔗 Quick navigation between test results
- 📊 Test execution summary

### HTML Benchmark Report Features
- 📈 Interactive Criterion charts
- ⚡ Container performance metrics
- 🔄 System workflow timings
- 📊 Comprehensive performance dashboard

## 🛠️ Development Workflow

1. **Write Code**: Implement features in `src/`
2. **Unit Tests**: Add tests in `src/` with `#[cfg(test)]`
3. **Integration Tests**: Add endpoint tests in `tests/integration_tests.rs`
4. **Benchmarks**: Add performance tests in `benches/`
5. **Run Tests**: `./scripts/run_tests.sh`
6. **Run Benchmarks**: `./scripts/run_benchmarks.sh`
7. **Review Reports**: Check HTML reports for insights

## 🔧 Customization

### Adding New Benchmarks
1. Add function to `benches/{service}_benchmark.rs`
2. Follow Criterion patterns for statistical accuracy
3. Use `black_box()` to prevent compiler optimizations

### Adding New Tests
1. **Unit**: Add to `src/` with `#[cfg(test)]`
2. **Integration**: Add to `tests/integration_tests.rs`
3. **System**: Add to `tests/system_integration_tests.rs`

### Benchmark Configuration
Criterion is configured for:
- Sample size: 100 iterations
- Measurement time: 10 seconds per benchmark
- HTML reports enabled with interactive charts

## 📊 Performance Targets

Based on our benchmarks, typical performance expectations:
- **Ingestion**: < 30 seconds per book
- **Indexing**: < 60 seconds per book
- **Search**: < 5 seconds response time
- **API Endpoints**: < 1 second response time

## 🚨 Troubleshooting

### Tests Failing
1. Ensure all services are running: `docker-compose up`
2. Check service health: `curl http://0.0.0.0:700{1,2,3}/status`
3. Review test logs in `test_results/`

### Benchmarks Not Running
1. Ensure Rust toolchain is up to date
2. Check Criterion dependencies in `Cargo.toml`
3. Verify `cargo bench` works individually

### Services Not Starting
1. Check Docker is running
2. Verify ports 7001-7003 are available
3. Check `docker-compose logs` for errors

---

🎯 **Goal**: Comprehensive testing ensures our SOA system is reliable, performant, and maintainable!