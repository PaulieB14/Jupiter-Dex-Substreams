#!/bin/bash

# JWT Test Script for Jupiter DEX Substreams
# This script tests the foundational store integration with JWT authentication

set -e

echo "🚀 Starting JWT Testing for Jupiter DEX Substreams..."

# Check if JWT token is set
if [ -z "$SUBSTREAMS_API_TOKEN" ]; then
    echo "❌ Error: SUBSTREAMS_API_TOKEN environment variable is not set"
    echo "Please set your JWT token:"
    echo "export SUBSTREAMS_API_TOKEN='your-jwt-token-here'"
    exit 1
fi

echo "✅ JWT Token found: ${SUBSTREAMS_API_TOKEN:0:20}..."

# Test 1: JWT Authentication Test
echo "🔐 Testing JWT Authentication..."
substreams run map_jwt_auth_test \
    --start-block 31310775 \
    --stop-block +10 \
    --manifest test/jwt_test.yaml

# Test 2: Foundational Store Access with JWT
echo "🏪 Testing Foundational Store Access with JWT..."
substreams run map_jwt_test \
    --start-block 31310775 \
    --stop-block +10 \
    --manifest test/jwt_test.yaml

# Test 3: Jupiter Instructions with JWT
echo "🔄 Testing Jupiter Instructions with JWT..."
substreams run map_jupiter_instructions \
    --start-block 31310775 \
    --stop-block +10 \
    --manifest substreams.yaml

# Test 4: Jupiter Analytics with JWT
echo "📊 Testing Jupiter Analytics with JWT..."
substreams run map_jupiter_analytics \
    --start-block 31310775 \
    --stop-block +10 \
    --manifest substreams.yaml

echo "✅ JWT Testing completed successfully!"

# Performance Test
echo "⚡ Running Performance Test..."
time substreams run map_jwt_test \
    --start-block 31310775 \
    --stop-block +100 \
    --manifest test/jwt_test.yaml

echo "🎉 All JWT tests passed!"