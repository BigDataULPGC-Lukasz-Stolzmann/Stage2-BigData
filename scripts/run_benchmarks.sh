#!/bin/bash
# ============================================================
#  Executes Criterion micro-benchmarks and container load tests
#  for all services, then generates HTML performance reports.
#  Outputs:
#    - Criterion results per service
#    - Container performance summaries
#    - End-to-end workflow benchmark
# ============================================================

# Create benchmark results directory
mkdir -p benchmark_results
mkdir -p benchmark_results/html_reports

echo "Starting comprehensive benchmark suite with HTML reports"
echo "================================================"

# Function to run service benchmarks and copy HTML reports
run_service_benchmarks() {
    local service=$1
    local benchmark_name=$2

    echo "Running $service benchmarks..."
    cd "services/$service"

    # Run benchmark
    cargo bench --bench "$benchmark_name"

    # Copy HTML reports to central location
    if [ -d "target/criterion" ]; then
        echo "📁 Copying HTML reports for $service..."
        cp -r target/criterion "../../benchmark_results/html_reports/${service}_criterion_reports"

        # Create an index file for easy navigation
        echo "<h2>$service Benchmark Results</h2>" > "../../benchmark_results/html_reports/${service}_index.html"
        echo "<ul>" >> "../../benchmark_results/html_reports/${service}_index.html"

        for dir in target/criterion/*/; do
            if [ -d "$dir" ]; then
                benchmark=$(basename "$dir")
                echo "<li><a href='${service}_criterion_reports/$benchmark/report/index.html'>$benchmark</a></li>" >> "../../benchmark_results/html_reports/${service}_index.html"
            fi
        done

        echo "</ul>" >> "../../benchmark_results/html_reports/${service}_index.html"
    fi

    cd ../..
}

# Function to run container performance tests
run_container_benchmarks() {
    local service=$1
    local port=$2

    echo "Running container performance tests for $service..."

    # Check if service is running
    if ! curl -s "http://0.0.0.0:$port/status" > /dev/null; then
        echo "⚠️  Warning: $service is not running on port $port"
        return 1
    fi

    # Run load tests using curl and measure response times
    echo "⚡ Load testing $service endpoints..."

    case $service in
        "ingestion-service")
            endpoints=("/status" "/ingest/list" "/ingest/status/1342")
            ;;
        "indexing-service")
            endpoints=("/status" "/index/status")
            ;;
        "search-service")
            endpoints=("/status" "/search?q=test" "/search?q=love&author=Test")
            ;;
    esac

    # Create simple performance report
    echo "<h2>$service Container Performance Results</h2>" > "benchmark_results/html_reports/${service}_container_performance.html"
    echo "<table border='1'><tr><th>Endpoint</th><th>Avg Response Time (ms)</th><th>Success Rate</th></tr>" >> "benchmark_results/html_reports/${service}_container_performance.html"

    for endpoint in "${endpoints[@]}"; do
        echo "Testing $endpoint..."
        total_time=0
        success_count=0
        total_requests=20

        for i in $(seq 1 $total_requests); do
            start_time=$(python3 -c "import time; print(int(time.time() * 1000))")
            if curl -s "http://0.0.0.0:$port$endpoint" > /dev/null 2>&1; then
                end_time=$(python3 -c "import time; print(int(time.time() * 1000))")
                response_time=$((end_time - start_time))
                total_time=$((total_time + response_time))
                success_count=$((success_count + 1))
            fi
            sleep 0.1
        done

        avg_time=$((total_time / total_requests))
        success_rate=$((success_count * 100 / total_requests))

        echo "<tr><td>$endpoint</td><td>$avg_time</td><td>$success_rate%</td></tr>" >> "benchmark_results/html_reports/${service}_container_performance.html"
    done

    echo "</table>" >> "benchmark_results/html_reports/${service}_container_performance.html"
}

# Function to run system-wide workflow benchmarks
run_system_benchmarks() {
    echo "Running system-wide workflow benchmarks..."

    echo "<h2>System-wide Workflow Performance</h2>" > "benchmark_results/html_reports/system_workflow_performance.html"
    echo "<table border='1'><tr><th>Book ID</th><th>Total Time (ms)</th><th>Ingest Time (ms)</th><th>Index Time (ms)</th><th>Search Time (ms)</th><th>Success</th></tr>" >> "benchmark_results/html_reports/system_workflow_performance.html"

    book_ids=("84" "11" "1342")

    for book_id in "${book_ids[@]}"; do
        echo "Testing full workflow for book $book_id..."

        start_time=$(python3 -c "import time; print(int(time.time() * 1000))")

        # Ingest
        ingest_start=$(python3 -c "import time; print(int(time.time() * 1000))")
        if curl -s -X POST "http://0.0.0.0:7001/ingest/$book_id" > /dev/null 2>&1; then
            ingest_end=$(python3 -c "import time; print(int(time.time() * 1000))")
            ingest_time=$((ingest_end - ingest_start))

            sleep 2  # Wait for ingestion

            # Index
            index_start=$(python3 -c "import time; print(int(time.time() * 1000))")
            if curl -s -X POST "http://0.0.0.0:7002/index/update/$book_id" > /dev/null 2>&1; then
                index_end=$(python3 -c "import time; print(int(time.time() * 1000))")
                index_time=$((index_end - index_start))

                sleep 3  # Wait for indexing

                # Search
                search_start=$(python3 -c "import time; print(int(time.time() * 1000))")
                if curl -s "http://0.0.0.0:7003/search?q=test" > /dev/null 2>&1; then
                    search_end=$(python3 -c "import time; print(int(time.time() * 1000))")
                    search_time=$((search_end - search_start))

                    total_time=$((search_end - start_time))
                    success="✅"
                else
                    total_time=0; ingest_time=0; index_time=0; search_time=0; success="❌"
                fi
            else
                total_time=0; ingest_time=0; index_time=0; search_time=0; success="❌"
            fi
        else
            total_time=0; ingest_time=0; index_time=0; search_time=0; success="❌"
        fi

        echo "<tr><td>$book_id</td><td>$total_time</td><td>$ingest_time</td><td>$index_time</td><td>$search_time</td><td>$success</td></tr>" >> "benchmark_results/html_reports/system_workflow_performance.html"
        sleep 1
    done

    echo "</table>" >> "benchmark_results/html_reports/system_workflow_performance.html"
}

# Create main index page
create_main_index() {
    echo "Creating main benchmark index..."

    cat > "benchmark_results/html_reports/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Big Data Search Engine - Benchmark Results</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .service-section { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }
        .criterion-link { display: inline-block; margin: 10px; padding: 10px 15px; background-color: #4CAF50; color: white; text-decoration: none; border-radius: 3px; }
        .container-link { display: inline-block; margin: 10px; padding: 10px 15px; background-color: #2196F3; color: white; text-decoration: none; border-radius: 3px; }
        .system-link { display: inline-block; margin: 10px; padding: 10px 15px; background-color: #FF9800; color: white; text-decoration: none; border-radius: 3px; }
    </style>
</head>
<body>
    <h1>Big Data Search Engine - Benchmark Results</h1>
    <p>Generated on: <strong>$(date)</strong></p>

    <div class="service-section">
        <h2>Individual Service Benchmarks (Criterion Reports)</h2>
        <a href="ingestion-service_index.html" class="criterion-link">Ingestion Service</a>
        <a href="indexing-service_index.html" class="criterion-link">Indexing Service</a>
        <a href="search-service_index.html" class="criterion-link">Search Service</a>
    </div>

    <div class="service-section">
        <h2>Container Performance Tests</h2>
        <a href="ingestion-service_container_performance.html" class="container-link">Ingestion Container</a>
        <a href="indexing-service_container_performance.html" class="container-link">Indexing Container</a>
        <a href="search-service_container_performance.html" class="container-link">Search Container</a>
    </div>

    <div class="service-section">
        <h2>System-wide Workflow</h2>
        <a href="system_workflow_performance.html" class="system-link">End-to-End Performance</a>
    </div>

    <div class="service-section">
        <h2>About These Reports</h2>
        <ul>
            <li><strong>Criterion Reports:</strong> Detailed micro-benchmarks with statistical analysis, charts, and performance trends</li>
            <li><strong>Container Performance:</strong> Real-world API endpoint response times under load</li>
            <li><strong>System Workflow:</strong> End-to-end performance measuring complete book processing pipeline</li>
        </ul>
    </div>
</body>
</html>
EOF
}

# Main execution
echo "Starting benchmark suite..."

# Run individual service benchmarks (Criterion HTML reports)
run_service_benchmarks "ingestion-service" "ingestion_benchmark"
run_service_benchmarks "indexing-service" "indexing_benchmark"
run_service_benchmarks "search-service" "search_benchmark"

# Run container benchmarks if services are running
run_container_benchmarks "ingestion-service" "7001"
run_container_benchmarks "indexing-service" "7002"
run_container_benchmarks "search-service" "7003"

# Run system-wide benchmarks
run_system_benchmarks

# Create main index
create_main_index

echo ""
echo "Benchmark suite completed!"
echo "Results location: benchmark_results/html_reports/"
echo "Open benchmark_results/html_reports/index.html in your browser"
echo "Criterion detailed reports are in subdirectories with interactive charts"