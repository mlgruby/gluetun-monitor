#!/usr/bin/env bash
# ASN Discovery Script for Gluetun Monitor
#
# This script helps you discover all possible ASNs used by your VPN provider
# by repeatedly restarting Gluetun and collecting unique ASN values.
#
# Usage:
#   1. Make sure gluetun-monitor is running
#   2. Update LXC_IP to your gluetun-monitor host
#   3. Run: ./scripts/discover-asns.sh
#
# The script will restart your docker-compose stack multiple times to connect
# to different VPN servers and collect their ASNs.

set -euo pipefail

# Configuration
LXC_IP="${GLUETUN_MONITOR_HOST:-localhost}"
STATUS_URL="http://${LXC_IP}:3010/status"

MIN_WAIT=20        # Minimum wait after bringing stack up
POLL_INTERVAL=5    # How often to poll status endpoint
MAX_WAIT=300       # Maximum wait for VPN to connect
ITERATIONS=20      # Number of times to restart and collect ASNs

declare -A ASNS

print_unique() {
  local joined
  joined="$(printf "%s\n" "${!ASNS[@]}" | sort | paste -sd, -)"
  if [[ -z "$joined" ]]; then
    joined="(none yet)"
  fi
  echo "  🔁 Unique ASNs so far: $joined"
}

echo "🔍 Discovering VPN Exit ASNs"
echo "--------------------------------"
echo "Monitor URL: $STATUS_URL"
echo "Iterations: $ITERATIONS"
echo

for i in $(seq 1 $ITERATIONS); do
  echo
  echo "▶ Iteration $i/$ITERATIONS"

  echo "Bringing stack down…"
  docker compose down

  echo "Bringing stack up…"
  docker compose up -d

  echo "Waiting minimum ${MIN_WAIT}s for VPN to stabilize…"
  sleep "$MIN_WAIT"

  echo "Polling for valid VPN status…"
  elapsed=0
  got_asn=""

  while [[ $elapsed -lt $MAX_WAIT ]]; do
    RESPONSE=$(curl -s --max-time 10 "$STATUS_URL" 2>/dev/null || echo "")

    # No response yet
    if [[ -z "$RESPONSE" ]]; then
      sleep "$POLL_INTERVAL"
      elapsed=$((elapsed + POLL_INTERVAL))
      continue
    fi

    # Endpoint returned an error (e.g., lookup API temporarily down)
    ERR=$(echo "$RESPONSE" | jq -r '.error // empty' 2>/dev/null)
    if [[ -n "$ERR" ]]; then
      echo "  …status error: $ERR"
      sleep "$POLL_INTERVAL"
      elapsed=$((elapsed + POLL_INTERVAL))
      continue
    fi

    ASN=$(echo "$RESPONSE" | jq -r '.asn // empty' 2>/dev/null)
    if [[ -n "$ASN" ]]; then
      IP=$(echo "$RESPONSE" | jq -r '.ip // empty')
      ORG=$(echo "$RESPONSE" | jq -r '.org // empty')

      ASNS["$ASN"]=1
      got_asn="$ASN"

      echo "  ✔ IP:  $IP"
      echo "  ✔ ASN: $ASN"
      echo "  ✔ Org: $ORG"
      break
    fi

    sleep "$POLL_INTERVAL"
    elapsed=$((elapsed + POLL_INTERVAL))
  done

  if [[ -z "$got_asn" ]]; then
    echo "  ✖ Timed out after ${MAX_WAIT}s waiting for VPN status"
  fi

  print_unique
done

echo
echo "✅ Discovery complete"
echo "--------------------------------"
echo "Distinct ASNs found:"
printf "%s\n" "${!ASNS[@]}" | sort | sed 's/^/  - /'

echo
echo "Add this to your .env file:"
echo -n "VPN_ALLOWED_ASNS="
printf "%s\n" "${!ASNS[@]}" | sort | paste -sd, -
echo
