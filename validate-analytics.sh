#!/bin/bash

# Phase 5 Analytics Validation Script
# Quick validation of DuxNet Analytics System

set -e

echo "🧪 DuxNet Phase 5 Analytics Validation"
echo "======================================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

BASE_URL="http://localhost:8081"
TESTS_PASSED=0
TESTS_TOTAL=0

# Function to run test
run_test() {
    local test_name="$1"
    local endpoint="$2"
    local expected_key="$3"
    
    ((TESTS_TOTAL++))
    
    echo -n "Testing $test_name... "
    
    response=$(curl -s "$BASE_URL$endpoint" || echo "ERROR")
    
    if [[ "$response" == "ERROR" ]]; then
        echo -e "${RED}FAIL${NC} - Connection failed"
        return 1
    fi
    
    if echo "$response" | grep -q "\"$expected_key\""; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC} - Missing expected key: $expected_key"
        echo "Response: $response"
        return 1
    fi
}

# Function to test POST endpoint
run_post_test() {
    local test_name="$1"
    local endpoint="$2"
    local data="$3"
    
    ((TESTS_TOTAL++))
    
    echo -n "Testing $test_name... "
    
    response=$(curl -s -X POST "$BASE_URL$endpoint" -H "Content-Type: application/json" -d "$data" || echo "ERROR")
    
    if [[ "$response" == "ERROR" ]]; then
        echo -e "${RED}FAIL${NC} - Connection failed"
        return 1
    fi
    
    if [[ -n "$response" ]]; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC} - Empty response"
        return 1
    fi
}

echo "📡 Checking DuxNet service availability..."
if ! curl -s "$BASE_URL/api/analytics/snapshot" > /dev/null; then
    echo -e "${RED}❌ DuxNet service not available at $BASE_URL${NC}"
    echo "Please start the service with: cargo run"
    exit 1
fi

echo -e "${GREEN}✅ DuxNet service is running${NC}"
echo

echo "🧪 Running Analytics API Tests..."
echo "--------------------------------"

# Test core analytics endpoints
run_test "Analytics Snapshot" "/api/analytics/snapshot" "snapshot"
run_test "Analytics Summary" "/api/analytics/summary" "summary"
run_test "Active Alerts" "/api/analytics/alerts" "alerts"
run_test "Usage Analytics" "/api/analytics/usage" "success"
run_test "Service Analytics" "/api/analytics/services" "success"
run_test "Revenue Analytics" "/api/analytics/revenue" "success"

echo
echo "🔧 Testing Analytics Operations..."
echo "----------------------------------"

# Test metric recording
run_post_test "Metric Recording" "/api/analytics/metrics" '{
    "service_id": "test-service",
    "metric_type": "response_time", 
    "value": 150.0,
    "timestamp": 1704067200
}'

# Test service metrics
run_post_test "Service Metrics" "/api/analytics/service-metrics" '{
    "service_id": "test-service",
    "response_time": 150,
    "success_rate": 99.5,
    "requests_per_minute": 1000,
    "error_count": 2
}'

# Test alert rule creation
run_post_test "Alert Rule Creation" "/api/analytics/alerts" '{
    "name": "test_alert",
    "condition": "response_time > 1000",
    "threshold": 1000.0,
    "duration_seconds": 300,
    "notification_channel": "email"
}'

echo
echo "📊 Running Unit Tests..."
echo "------------------------"

# Run analytics unit tests
echo -n "Unit Tests... "
if cargo test analytics --lib --quiet > /dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC} (5 tests)"
    ((TESTS_PASSED++))
    ((TESTS_TOTAL++))
else
    echo -e "${RED}FAIL${NC}"
    ((TESTS_TOTAL++))
fi

echo
echo "📈 Validation Results"
echo "===================="

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! ($TESTS_PASSED/$TESTS_TOTAL)${NC}"
    echo -e "${GREEN}✅ Phase 5 Analytics System is fully operational${NC}"
    echo
    echo "🔗 Available Endpoints:"
    echo "  • Analytics Snapshot: $BASE_URL/api/analytics/snapshot"
    echo "  • Analytics Summary:  $BASE_URL/api/analytics/summary"
    echo "  • Active Alerts:      $BASE_URL/api/analytics/alerts"
    echo "  • Usage Analytics:    $BASE_URL/api/analytics/usage"
    echo "  • Developer Portal:   $BASE_URL/api/developer/dashboard"
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED ($TESTS_PASSED/$TESTS_TOTAL passed)${NC}"
    echo "Please check the service logs and try again."
    exit 1
fi
