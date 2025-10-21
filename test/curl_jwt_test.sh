#!/bin/bash

# JWT Testing with curl commands for Jupiter DEX Substreams
# This script tests the foundational store integration using curl
# No need to commit JWT tokens - they're passed as environment variables

set -e

echo "🚀 Starting JWT Testing with curl for Jupiter DEX Substreams..."

# Check if JWT token is set
if [ -z "$SUBSTREAMS_API_TOKEN" ]; then
    echo "❌ Error: SUBSTREAMS_API_TOKEN environment variable is not set"
    echo "Please set your JWT token:"
    echo "export SUBSTREAMS_API_TOKEN='your-jwt-token-here'"
    exit 1
fi

echo "✅ JWT Token found: ${SUBSTREAMS_API_TOKEN:0:20}..."

# Base URL for substreams.dev
BASE_URL="https://substreams.dev"

# Test 1: Test JWT Authentication
echo "🔐 Testing JWT Authentication..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jwt_auth_test"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ JWT Authentication test completed"

# Test 2: Test Foundational Store Access
echo "🏪 Testing Foundational Store Access..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jwt_test"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ Foundational Store Access test completed"

# Test 3: Test Jupiter Instructions
echo "🔄 Testing Jupiter Instructions..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jupiter_instructions"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ Jupiter Instructions test completed"

# Test 4: Test Jupiter Analytics
echo "📊 Testing Jupiter Analytics..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jupiter_analytics"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ Jupiter Analytics test completed"

# Test 5: Performance Test
echo "⚡ Running Performance Test..."
time curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310875,
    "modules": ["map_jwt_test"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ Performance test completed"

# Test 6: Test SPL Account Store
echo "🔑 Testing SPL Account Store..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_spl_initialized_account"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ SPL Account Store test completed"

# Test 7: Test Trading Data Store
echo "📈 Testing Trading Data Store..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jupiter_trading_data"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ Trading Data Store test completed"

# Test 8: Test Token Price Store
echo "💰 Testing Token Price Store..."
curl -X POST \
  -H "Authorization: Bearer $SUBSTREAMS_API_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_token_prices"]
  }' \
  "$BASE_URL/api/v1/run" \
  --silent --show-error --fail

echo "✅ Token Price Store test completed"

echo "🎉 All JWT tests completed successfully!"
echo "📊 Summary:"
echo "  - JWT Authentication: ✅"
echo "  - Foundational Store Access: ✅"
echo "  - Jupiter Instructions: ✅"
echo "  - Jupiter Analytics: ✅"
echo "  - Performance: ✅"
echo "  - SPL Account Store: ✅"
echo "  - Trading Data Store: ✅"
echo "  - Token Price Store: ✅"