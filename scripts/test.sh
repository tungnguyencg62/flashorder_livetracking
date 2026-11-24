#!/bin/bash
set -euo pipefail

# Nạp toàn bộ biến trong .env vào shell hiện tại
set -a
source .env
set +a

EMAIL="smoketest-1795356994@example.com"
PASSWORD="SmokeTest123!"


LOGIN=$(curl -s -X POST "http://127.0.0.1:9999/token?grant_type=password" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\"}")
ACCESS_TOKEN=$(echo "$LOGIN" | jq -r .access_token)

curl -H "Authorization: Bearer $ACCESS_TOKEN" localhost:3000/v1/merchant-only