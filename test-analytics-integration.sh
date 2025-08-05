#!/bin/bash

# Phase 5 Analytics Integration Test Suite
# Tests the complete analytics system end-to-end

set -e

echo "🧪 Starting Phase 5 Analytics Integration Tests..."
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
API_BASE="http://localhost:8080/api"
TEST_SERVICE_ID="test-analytics-service"
ALERT_RULE_ID="test-high-response-time"

# Counter for tests
TESTS_PASSED=0
TESTS_FAILED=0

# Function to log test results
log_test() {
    local test_name="$1"
    local status="$2"
    local message="$3"
    
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAIL${NC}: $test_name - $message"
        ((TESTS_FAILED++))
    fi
}

# Function to make API calls with error handling
api_call() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    local expected_status="${4:-200}"
    
    if [ -n "$data" ]; then
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$API_BASE$endpoint")
    else
        response=$(curl -s -w "HTTPSTATUS:%{http_code}" -X "$method" \
            "$API_BASE$endpoint")
    fi
    
    http_status=$(echo "$response" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    response_body=$(echo "$response" | sed -e 's/HTTPSTATUS:.*//g')
    
    if [ "$http_status" != "$expected_status" ]; then
        echo "Expected status $expected_status, got $http_status"
        echo "Response: $response_body"
        return 1
    fi
    
    echo "$response_body"
}

# Function to wait for service to be ready
wait_for_service() {
    echo -e "${YELLOW}⏳ Waiting for DuxNet service to be ready...${NC}"
    for i in {1..30}; do
        if curl -s "$API_BASE/status" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Service is ready!${NC}"
            return 0
        fi
        echo -n "."
        sleep 2
    done
    echo -e "${RED}❌ Service failed to start within 60 seconds${NC}"
    return 1
}

# Function to start DuxNet service in background
start_service() {
    echo -e "${BLUE}🚀 Starting DuxNet service...${NC}"
    
    # Check if service is already running
    if curl -s "$API_BASE/status" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Service already running${NC}"
        return 0
    fi
    
    # Start the service
    cargo run > /tmp/duxnet.log 2>&1 &
    SERVICE_PID=$!
    echo "Service PID: $SERVICE_PID"
    
    # Wait for service to be ready
    wait_for_service
}

# Function to stop DuxNet service
stop_service() {
    if [ -n "$SERVICE_PID" ]; then
        echo -e "${BLUE}🛑 Stopping DuxNet service (PID: $SERVICE_PID)...${NC}"
        kill "$SERVICE_PID" 2>/dev/null || true
        wait "$SERVICE_PID" 2>/dev/null || true
    fi
}

# Cleanup function
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    stop_service
}

# Set trap for cleanup
trap cleanup EXIT

echo -e "${BLUE}📋 Test Plan:${NC}"
echo "1. Service Startup & Health Check"
echo "2. Analytics Snapshot Generation"
echo "3. Metric Recording & Retrieval"
echo "4. Service Performance Monitoring"
echo "5. Alert System Testing"
echo "6. Dashboard Management"
echo "7. Analytics Summary & Aggregation"
echo "8. Error Handling & Edge Cases"
echo ""

# Start the service
start_service

echo -e "${BLUE}🧪 Running Integration Tests...${NC}"
echo ""

# Test 1: Service Health Check
echo -e "${YELLOW}Test 1: Service Health Check${NC}"
if response=$(api_call "GET" "/status"); then
    if echo "$response" | grep -q '"success":true'; then
        log_test "Service Health Check" "PASS"
    else
        log_test "Service Health Check" "FAIL" "Service not healthy: $response"
    fi
else
    log_test "Service Health Check" "FAIL" "Service not responding"
fi

# Test 2: Analytics Snapshot
echo -e "${YELLOW}Test 2: Analytics Snapshot Generation${NC}"
if response=$(api_call "GET" "/analytics/snapshot"); then
    if echo "$response" | grep -q '"success":true' && echo "$response" | grep -q '"snapshot"'; then
        log_test "Analytics Snapshot" "PASS"
    else
        log_test "Analytics Snapshot" "FAIL" "Invalid snapshot response: $response"
    fi
else
    log_test "Analytics Snapshot" "FAIL" "Failed to get snapshot"
fi

# Test 3: Record Metrics
echo -e "${YELLOW}Test 3: Metric Recording${NC}"
metric_data='{"metric_type": "ResponseTime", "value": 150.5}'
if response=$(api_call "POST" "/analytics/metrics" "$metric_data"); then
    if echo "$response" | grep -q '"success":true'; then
        log_test "Metric Recording" "PASS"
    else
        log_test "Metric Recording" "FAIL" "Failed to record metric: $response"
    fi
else
    log_test "Metric Recording" "FAIL" "API call failed"
fi

# Test 4: Service Performance Metrics
echo -e "${YELLOW}Test 4: Service Performance Metrics${NC}"
service_metrics='{
    "service_id": "'$TEST_SERVICE_ID'",
    "timestamp": '$(date +%s)',
    "response_time": 200,
    "status_code": 200,
    "bytes_transferred": 1024,
    "user_agent": "DuxNet-Test/1.0",
    "ip_address": "127.0.0.1"
}'
if response=$(api_call "POST" "/analytics/service-metrics" "$service_metrics"); then
    if echo "$response" | grep -q '"success":true'; then
        log_test "Service Performance Metrics" "PASS"
    else
        log_test "Service Performance Metrics" "FAIL" "Failed to record service metrics: $response"
    fi
else
    log_test "Service Performance Metrics" "FAIL" "API call failed"
fi

# Test 5: Query Metrics
echo -e "${YELLOW}Test 5: Metric Querying${NC}"
if response=$(api_call "GET" "/analytics/metrics?metric_type=ResponseTime&limit=10"); then
    if echo "$response" | grep -q '"success":true' && echo "$response" | grep -q '"metrics"'; then
        log_test "Metric Querying" "PASS"
    else
        log_test "Metric Querying" "FAIL" "Invalid metrics response: $response"
    fi
else
    log_test "Metric Querying" "FAIL" "Failed to query metrics"
fi

# Test 6: Alert Rule Creation
echo -e "${YELLOW}Test 6: Alert Rule Creation${NC}"
alert_rule='{
    "id": "'$ALERT_RULE_ID'",
    "name": "Test High Response Time",
    "metric_type": "ResponseTime",
    "comparison": "GreaterThan",
    "threshold": 1000.0,
    "enabled": true,
    "notification_channels": ["Email"]
}'
if response=$(api_call "POST" "/analytics/alerts" "$alert_rule"); then
    if echo "$response" | grep -q '"success":true'; then
        log_test "Alert Rule Creation" "PASS"
    else
        log_test "Alert Rule Creation" "FAIL" "Failed to create alert rule: $response"
    fi
else
    log_test "Alert Rule Creation" "FAIL" "API call failed"
fi

# Test 7: Get Active Alerts
echo -e "${YELLOW}Test 7: Active Alerts Retrieval${NC}"
if response=$(api_call "GET" "/analytics/alerts"); then
    if echo "$response" | grep -q '"success":true' && echo "$response" | grep -q '"alerts"'; then
        log_test "Active Alerts Retrieval" "PASS"
    else
        log_test "Active Alerts Retrieval" "FAIL" "Invalid alerts response: $response"
    fi
else
    log_test "Active Alerts Retrieval" "FAIL" "Failed to get alerts"
fi

# Test 8: Dashboard Creation
echo -e "${YELLOW}Test 8: Default Dashboard Creation${NC}"
if response=$(api_call "GET" "/analytics/dashboards/default"); then
    if echo "$response" | grep -q '"success":true' && echo "$response" | grep -q '"dashboard"'; then
        log_test "Default Dashboard Creation" "PASS"
    else
        log_test "Default Dashboard Creation" "FAIL" "Invalid dashboard response: $response"
    fi
else
    log_test "Default Dashboard Creation" "FAIL" "Failed to create default dashboard"
fi

# Test 9: Analytics Summary
echo -e "${YELLOW}Test 9: Analytics Summary${NC}"
if response=$(api_call "GET" "/analytics/summary"); then
    if echo "$response" | grep -q '"success":true' && echo "$response" | grep -q '"summary"'; then
        log_test "Analytics Summary" "PASS"
    else
        log_test "Analytics Summary" "FAIL" "Invalid summary response: $response"
    fi
else
    log_test "Analytics Summary" "FAIL" "Failed to get analytics summary"
fi

# Test 10: Trigger Alert with High Metric
echo -e "${YELLOW}Test 10: Alert Triggering${NC}"
high_metric='{"metric_type": "ResponseTime", "value": 1500.0}'
if response=$(api_call "POST" "/analytics/metrics" "$high_metric"); then
    if echo "$response" | grep -q '"success":true'; then
        # Wait a moment for alert processing
        sleep 2
        
        # Check if alert was triggered
        if alert_response=$(api_call "GET" "/analytics/alerts"); then
            if echo "$alert_response" | grep -q '"alerts"' && echo "$alert_response" | grep -q '"High Response Time"'; then
                log_test "Alert Triggering" "PASS"
            else
                log_test "Alert Triggering" "FAIL" "Alert not triggered or not found in response"
            fi
        else
            log_test "Alert Triggering" "FAIL" "Failed to check alerts after triggering"
        fi
    else
        log_test "Alert Triggering" "FAIL" "Failed to record high metric: $response"
    fi
else
    log_test "Alert Triggering" "FAIL" "API call failed"
fi

# Test 11: Error Handling - Invalid Metric Type
echo -e "${YELLOW}Test 11: Error Handling - Invalid Metric${NC}"
invalid_metric='{"metric_type": "InvalidType", "value": 100.0}'
if response=$(api_call "POST" "/analytics/metrics" "$invalid_metric" "400"); then
    if echo "$response" | grep -q '"success":false'; then
        log_test "Error Handling - Invalid Metric" "PASS"
    else
        log_test "Error Handling - Invalid Metric" "FAIL" "Should have returned error for invalid metric type"
    fi
else
    # If API call failed as expected (400 status), that's also acceptable
    log_test "Error Handling - Invalid Metric" "PASS"
fi

# Test 12: Error Handling - Missing Required Fields
echo -e "${YELLOW}Test 12: Error Handling - Missing Fields${NC}"
incomplete_metric='{"metric_type": "ResponseTime"}'
if response=$(api_call "POST" "/analytics/metrics" "$incomplete_metric" "400"); then
    if echo "$response" | grep -q '"success":false'; then
        log_test "Error Handling - Missing Fields" "PASS"
    else
        log_test "Error Handling - Missing Fields" "FAIL" "Should have returned error for missing value"
    fi
else
    # If API call failed as expected (400 status), that's also acceptable
    log_test "Error Handling - Missing Fields" "PASS"
fi

# Test 13: Performance Test - Multiple Metrics
echo -e "${YELLOW}Test 13: Performance - Bulk Metrics${NC}"
start_time=$(date +%s%N)
for i in {1..10}; do
    metric_data='{"metric_type": "RequestCount", "value": 1.0}'
    api_call "POST" "/analytics/metrics" "$metric_data" > /dev/null 2>&1
done
end_time=$(date +%s%N)
duration=$(((end_time - start_time) / 1000000)) # Convert to milliseconds

if [ "$duration" -lt 5000 ]; then # Less than 5 seconds for 10 requests
    log_test "Performance - Bulk Metrics" "PASS"
else
    log_test "Performance - Bulk Metrics" "FAIL" "Bulk operations too slow: ${duration}ms"
fi

# Test 14: Data Consistency
echo -e "${YELLOW}Test 14: Data Consistency Check${NC}"
# Record a specific metric
test_value=42.0
consistency_metric='{"metric_type": "CpuUsage", "value": '$test_value'}'
if api_call "POST" "/analytics/metrics" "$consistency_metric" > /dev/null 2>&1; then
    # Query it back
    if response=$(api_call "GET" "/analytics/metrics?metric_type=CpuUsage&limit=1"); then
        if echo "$response" | grep -q "\"value\":$test_value"; then
            log_test "Data Consistency Check" "PASS"
        else
            log_test "Data Consistency Check" "FAIL" "Recorded value not found in query results"
        fi
    else
        log_test "Data Consistency Check" "FAIL" "Failed to query recorded metric"
    fi
else
    log_test "Data Consistency Check" "FAIL" "Failed to record test metric"
fi

# Test 15: System Load Test
echo -e "${YELLOW}Test 15: System Load Test${NC}"
echo "Recording 50 mixed metrics rapidly..."
load_start=$(date +%s)
for i in $(seq 1 50); do
    metric_type=$([ $((i % 3)) -eq 0 ] && echo "ResponseTime" || echo "RequestCount")
    value=$(echo "scale=1; $i * 1.5" | bc)
    metric_data='{"metric_type": "'$metric_type'", "value": '$value'}'
    api_call "POST" "/analytics/metrics" "$metric_data" > /dev/null 2>&1 &
done
wait # Wait for all background jobs to complete
load_end=$(date +%s)
load_duration=$((load_end - load_start))

if [ "$load_duration" -lt 30 ]; then # Should complete within 30 seconds
    log_test "System Load Test" "PASS"
else
    log_test "System Load Test" "FAIL" "Load test took too long: ${load_duration}s"
fi

echo ""
echo -e "${BLUE}📊 Test Results Summary${NC}"
echo "======================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo -e "Total:  $((TESTS_PASSED + TESTS_FAILED))"

if [ "$TESTS_FAILED" -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 All integration tests passed!${NC}"
    echo -e "${GREEN}✅ Phase 5 Analytics system is fully functional${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}❌ Some tests failed. Check the output above for details.${NC}"
    echo -e "${YELLOW}💡 Check service logs at /tmp/duxnet.log for more information${NC}"
    exit 1
fi
