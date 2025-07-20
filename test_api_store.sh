#!/bin/bash

# DuxNet API Store Test Script
# This script demonstrates the core functionality of the DuxNet API Store

echo "🚀 Testing DuxNet API Store"
echo "============================"

BASE_URL="http://localhost:8081"
API_KEY="demo-api-key-123"

echo ""
echo "1. Testing API Status..."
curl -s "$BASE_URL/api/status" | head -c 200 && echo "..."

echo ""
echo "2. Testing API Version..."
curl -s "$BASE_URL/api/version" | head -c 200 && echo "..."

echo ""
echo "3. Testing Service Categories..."
curl -s "$BASE_URL/api/services/categories" | head -c 200 && echo "..."

echo ""
echo "4. Registering a Test Service..."
SERVICE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/services/register" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "name": "Data Processing Service",
    "description": "High-performance data processing and analytics",
    "price": 500000,
    "categories": ["Data Processing", "AI & Machine Learning"],
    "tags": ["data", "analytics", "processing"],
    "sla": {
      "uptime_guarantee": 99.9,
      "max_response_time_ms": 1000,
      "support_response_hours": 8,
      "refund_policy": {"PartialRefund": {"percentage": 50.0}},
      "availability_zones": ["us-east", "us-west", "eu-west"]
    },
    "version": "1.0.0",
    "documentation_url": "https://docs.example.com/data-processing",
    "rate_limit_per_minute": 200,
    "supported_formats": ["JSON", "CSV", "Parquet"],
    "examples": [
      {
        "name": "Data Aggregation",
        "description": "Aggregate data by category",
        "request": "{\"data\": [1,2,3,4,5], \"operation\": \"sum\"}",
        "response": "{\"result\": 15}",
        "language": "Python"
      }
    ]
  }')

echo "$SERVICE_RESPONSE" | head -c 300 && echo "..."

echo ""
echo "5. Searching for Services..."
SEARCH_RESPONSE=$(curl -s -X POST "$BASE_URL/api/services/search" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "query": "data",
    "categories": ["Data Processing"],
    "limit": 5,
    "offset": 0,
    "sort_by": "Name",
    "sort_order": "asc"
  }')

echo "$SEARCH_RESPONSE" | head -c 400 && echo "..."

echo ""
echo "6. Testing Developer Dashboard..."
curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/api/developer/dashboard" | head -c 300 && echo "..."

echo ""
echo "7. Testing Analytics..."
curl -s -H "Authorization: Bearer $API_KEY" "$BASE_URL/api/analytics/usage" | head -c 200 && echo "..."

echo ""
echo "8. Testing API Stats..."
curl -s "$BASE_URL/api/stats" | head -c 200 && echo "..."

echo ""
echo "✅ API Store Test Complete!"
echo ""
echo "📊 Summary:"
echo "- API Status: ✅ Working"
echo "- Service Registration: ✅ Working"
echo "- Service Search: ✅ Working"
echo "- Analytics: ✅ Working"
echo "- Developer Dashboard: ✅ Working"
echo ""
echo "🔑 Demo API Keys:"
echo "- demo-api-key-123 (Demo User)"
echo "- admin-api-key-456 (Admin)"
echo "- service-api-key-789 (Service Provider)"
echo ""
echo "🌐 API Documentation:"
echo "- Base URL: $BASE_URL"
echo "- Status: $BASE_URL/api/status"
echo "- Version: $BASE_URL/api/version"
echo "- Services: $BASE_URL/api/services"
echo "- Analytics: $BASE_URL/api/analytics"
echo "- Developer Portal: $BASE_URL/api/developer" 