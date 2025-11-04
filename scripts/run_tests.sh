#!/bin/bash
# ============================================================
#  Runs unit, integration, and system tests across all services,
#  then generates an HTML test summary report.
#  Outputs:
#    - Per-service test logs
#    - System integration results
#    - Combined HTML test report
# ==========================================================

echo "Starting comprehensive test suite"
echo "===================================="

# Create test results directory
mkdir -p test_results

# Function to run service tests
run_service_tests() {
    local service=$1
    echo "Testing $service..."

    cd "services/$service"

    # Run unit tests (skip if no lib target)
    echo "  Running unit tests..."
    if cargo test --lib --dry-run > /dev/null 2>&1; then
        cargo test --lib > "../test_results/${service}_unit_tests.log" 2>&1
        unit_exit_code=$?
    else
        echo "No library targets found, skipping unit tests" > "../test_results/${service}_unit_tests.log"
        unit_exit_code=0
    fi

    # Run integration tests
    echo "  Running integration tests..."
    cargo test --test integration_tests > "../test_results/${service}_integration_tests.log" 2>&1
    integration_exit_code=$?

    cd ../..

    if [ $unit_exit_code -eq 0 ]; then
        if grep -q "No library targets found" "../test_results/${service}_unit_tests.log"; then
            echo "  ⏭️ Unit tests skipped (binary crate)"
        else
            echo "  ✅ Unit tests passed"
        fi
    else
        echo "  ❌ Unit tests failed (see test_results/${service}_unit_tests.log)"
    fi

    if [ $integration_exit_code -eq 0 ]; then
        echo "  ✅ Integration tests passed"
    else
        echo "  ❌ Integration tests failed (see test_results/${service}_integration_tests.log)"
    fi

    return $((unit_exit_code + integration_exit_code))
}

# Function to check if services are running for integration tests
check_services() {
    echo "Checking if services are running..."

    services=(
        "http://0.0.0.0:7001/status ingestion"
        "http://0.0.0.0:7002/status indexing"
        "http://0.0.0.0:7003/status search"
    )

    all_running=true
    for service_info in "${services[@]}"; do
        url=$(echo "$service_info" | cut -d' ' -f1)  # Get URL part
        name=$(echo "$service_info" | cut -d' ' -f2) # Get name part

        if curl -s "$url" > /dev/null 2>&1; then
            echo "  ✅ $name service is running"
        else
            echo "  ❌ $name service is not running"
            all_running=false
        fi
    done

    if [ "$all_running" = false ]; then
        echo ""
        echo "⚠️  Some services are not running. Start them with:"
        echo "   docker-compose up --build"
        echo ""
        echo "Or run individual services:"
        echo "   cd ingestion-service && cargo run"
        echo "   cd indexing-service && cargo run"
        echo "   cd search-service && cargo run"
        echo ""
        return 1
    fi

    return 0
}

# Function to run system integration tests
run_system_tests() {
    echo "Running system integration tests..."

    cargo test --test system_integration_tests > "test_results/system_integration_tests.log" 2>&1
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "  ✅ System integration tests passed"
    else
        echo "  ❌ System integration tests failed (see test_results/system_integration_tests.log)"
        echo "  💡 Make sure all services are running: docker-compose up"
    fi

    return $exit_code
}

# Function to generate test report
generate_test_report() {
    echo "Generating test report..."

    cat > "test_results/test_report.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Test Results - Big Data Search Engine</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        .test-section { margin: 20px 0; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }
        .pass { color: #4CAF50; font-weight: bold; }
        .fail { color: #f44336; font-weight: bold; }
        .log-link { display: inline-block; margin: 5px; padding: 5px 10px; background-color: #2196F3; color: white; text-decoration: none; border-radius: 3px; font-size: 12px; }
        pre { background-color: #f5f5f5; padding: 10px; border-radius: 3px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>Test Results - Big Data Search Engine</h1>
    <p>Generated on: <strong>$(date)</strong></p>
EOF

    # Add individual service test results
    echo "<div class='test-section'><h2>Individual Service Tests</h2>" >> "test_results/test_report.html"

    for service in "ingestion-service" "indexing-service" "search-service"; do
        echo "<h3>$service</h3>" >> "test_results/test_report.html"

        if [ -f "test_results/${service}_unit_tests.log" ]; then
            if grep -q "test result: ok" "test_results/${service}_unit_tests.log"; then
                echo "<p class='pass'>✅ Unit Tests: PASSED</p>" >> "test_results/test_report.html"
            else
                echo "<p class='fail'>❌ Unit Tests: FAILED</p>" >> "test_results/test_report.html"
            fi
            echo "<a href='${service}_unit_tests.log' class='log-link'>Unit Test Log</a>" >> "test_results/test_report.html"
        fi

        if [ -f "test_results/${service}_integration_tests.log" ]; then
            if grep -q "test result: ok" "test_results/${service}_integration_tests.log"; then
                echo "<p class='pass'>✅ Integration Tests: PASSED</p>" >> "test_results/test_report.html"
            else
                echo "<p class='fail'>❌ Integration Tests: FAILED</p>" >> "test_results/test_report.html"
            fi
            echo "<a href='${service}_integration_tests.log' class='log-link'>Integration Test Log</a>" >> "test_results/test_report.html"
        fi
    done

    echo "</div>" >> "test_results/test_report.html"

    # Add system test results
    echo "<div class='test-section'><h2>System Integration Tests</h2>" >> "test_results/test_report.html"

    if [ -f "test_results/system_integration_tests.log" ]; then
        if grep -q "test result: ok" "test_results/system_integration_tests.log"; then
            echo "<p class='pass'>✅ System Integration Tests: PASSED</p>" >> "test_results/test_report.html"
        else
            echo "<p class='fail'>❌ System Integration Tests: FAILED</p>" >> "test_results/test_report.html"
        fi
        echo "<a href='system_integration_tests.log' class='log-link'>System Test Log</a>" >> "test_results/test_report.html"
    fi

    echo "</div>" >> "test_results/test_report.html"

    # Add instructions
    echo "<div class='test-section'>" >> "test_results/test_report.html"
    echo "<h2>Instructions</h2>" >> "test_results/test_report.html"
    echo "<ul>" >> "test_results/test_report.html"
    echo "<li><strong>Unit Tests:</strong> Test individual functions and modules in isolation</li>" >> "test_results/test_report.html"
    echo "<li><strong>Integration Tests:</strong> Test API endpoints and service interactions</li>" >> "test_results/test_report.html"
    echo "<li><strong>System Tests:</strong> Test complete end-to-end workflows</li>" >> "test_results/test_report.html"
    echo "</ul>" >> "test_results/test_report.html"
    echo "<p>To run tests manually:</p>" >> "test_results/test_report.html"
    echo "<pre>./scripts/run_tests.sh</pre>" >> "test_results/test_report.html"
    echo "</div>" >> "test_results/test_report.html"

    echo "</body></html>" >> "test_results/test_report.html"
}

# Main execution
total_failures=0

echo "Starting test suite..."
echo ""

# Run individual service tests
run_service_tests "ingestion-service"
total_failures=$((total_failures + $?))

run_service_tests "indexing-service"
total_failures=$((total_failures + $?))

run_service_tests "search-service"
total_failures=$((total_failures + $?))

echo ""

# Check if services are running for system tests
if check_services; then
    echo ""
    run_system_tests
    total_failures=$((total_failures + $?))
else
    echo "Skipping system integration tests (services not running)"
    echo ""
fi

# Generate report
generate_test_report

echo ""
echo "Test Summary:"
if [ $total_failures -eq 0 ]; then
    echo "✅ All tests passed!"
else
    echo "❌ $total_failures test suite(s) failed"
fi

echo ""
echo "Results saved in test_results/ directory"
echo "Open test_results/test_report.html for detailed results"

exit $total_failures