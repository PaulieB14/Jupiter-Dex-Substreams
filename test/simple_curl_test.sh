#!/bin/bash

# Simple curl test for Jupiter DEX Substreams with JWT
# Just run this script with your JWT token

JWT_TOKEN="eyJhbGciOiJLTVNFUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3OTcwNTIxNDMsImp0aSI6IjA5NTQxZmQ5LTBmM2UtNDdmZS04MDZlLTdmZWM3YzIzOGMzMSIsImlhdCI6MTc2MTA1MjE0MywiaXNzIjoiZGZ1c2UuaW8iLCJzdWIiOiIwYm9qaTQ5NTUyMjg5MjIwYzVkYjciLCJ2IjoyLCJha2kiOiIwNjcwOTYwYmNhOGM5YTUzNDdhODQzM2VhODc1YmVmODIzMGRiNjVmMmY2NDJkNWFlMGZiYmZkODZhNmM1OTlmIiwidWlkIjoiMGJvamk0OTU1MjI4OTIyMGM1ZGI3Iiwic3Vic3RyZWFtc19wbGFuX3RpZXIiOiJGUkVFIiwiY2ZnIjp7IlNVQlNUUkVBTVNfTUFYX1JFUVVFU1RTIjoiMiIsIlNVQlNUUkVBTVNfUEFSQUxMRUxfSk9CUyI6IjUiLCJTVUJTVFJFQU1TX1BBUkFMTEVMX1dPUktFUlMiOiI1In19.iUq-RHMIqznaWSKDUkmJa-Lj6RtfRAhflgwLtF554C-pyaKxLBN65aqswsG2np_Ldov_itT9wpnAr7IYkofwRg"

echo "🚀 Testing Jupiter DEX Substreams with JWT..."

# Test 1: JWT Authentication
echo "🔐 Testing JWT Authentication..."
curl -X POST \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jwt_auth_test"]
  }' \
  "https://substreams.dev/api/v1/run" \
  --silent --show-error --fail

echo "✅ JWT Authentication test completed"

# Test 2: Foundational Store Access
echo "🏪 Testing Foundational Store Access..."
curl -X POST \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jwt_test"]
  }' \
  "https://substreams.dev/api/v1/run" \
  --silent --show-error --fail

echo "✅ Foundational Store Access test completed"

# Test 3: Jupiter Instructions
echo "🔄 Testing Jupiter Instructions..."
curl -X POST \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jupiter_instructions"]
  }' \
  "https://substreams.dev/api/v1/run" \
  --silent --show-error --fail

echo "✅ Jupiter Instructions test completed"

# Test 4: Jupiter Analytics
echo "📊 Testing Jupiter Analytics..."
curl -X POST \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "start_block": 31310775,
    "stop_block": 31310785,
    "modules": ["map_jupiter_analytics"]
  }' \
  "https://substreams.dev/api/v1/run" \
  --silent --show-error --fail

echo "✅ Jupiter Analytics test completed"

echo "🎉 All JWT tests completed successfully!"