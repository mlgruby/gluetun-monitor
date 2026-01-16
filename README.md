# Gluetun Monitor

A lightweight VPN health monitoring service written in Rust that integrates with [Gluetun](https://github.com/qdm12/gluetun) to monitor VPN connections, validate ASNs, detect server changes, and send notifications. Designed for seamless integration with [Uptime Kuma](https://github.com/louislam/uptime-kuma) and other monitoring tools.

## Features

- **VPN Health Monitoring**: Continuously monitors your VPN connection status
- **ASN Validation**: Ensures your IP address belongs to allowed Autonomous System Numbers
- **Change Detection**: Detects and notifies when VPN server changes (IP, country, ASN)
- **Multiple IP Lookup Methods**: Supports Gluetun API, ifconfig.co, and ip-api.com
- **Port Forwarding Status**: Monitors port forwarding configuration
- **Flexible Notifications**: Sends alerts via [ntfy](https://ntfy.sh) for status changes
- **HTTP API**: Provides `/status` and `/check` endpoints for health checks
- **Lightweight**: Built with Rust for minimal resource usage

## Quick Start

### Using Docker Compose (Recommended)

```yaml
version: '3.8'

services:
  gluetun:
    image: qmcgaw/gluetun
    container_name: gluetun
    cap_add:
      - NET_ADMIN
    environment:
      - VPN_SERVICE_PROVIDER=your_provider
      - VPN_TYPE=openvpn
      # Add your VPN configuration here
    ports:
      - "8888:8888/tcp" # HTTP proxy
      - "8388:8388/tcp" # Shadowsocks
    restart: unless-stopped

  gluetun-monitor:
    image: ghcr.io/yourusername/gluetun-monitor:latest
    container_name: gluetun-monitor
    network_mode: "service:gluetun"
    environment:
      - VPN_ALLOWED_ASNS=AS12345,AS67890
      - GLUETUN_API_URL=http://localhost:8000
      - NTFY_URL=https://ntfy.sh/your-topic
      - NTFY_INTERVAL_HOURS=2
      - VPN_CHECK_INTERVAL_MINUTES=5
    depends_on:
      - gluetun
    restart: unless-stopped
```

### Using Docker

```bash
docker run -d \
  --name gluetun-monitor \
  --network container:gluetun \
  -e VPN_ALLOWED_ASNS=AS12345,AS67890 \
  -e GLUETUN_API_URL=http://localhost:8000 \
  -e NTFY_URL=https://ntfy.sh/your-topic \
  ghcr.io/yourusername/gluetun-monitor:latest
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/gluetun-monitor.git
cd gluetun-monitor

# Build with Cargo
cargo build --release

# Optional: Install development tools
cargo install cargo-audit cargo-watch

# Run
VPN_ALLOWED_ASNS=AS12345,AS67890 \
GLUETUN_API_URL=http://localhost:8000 \
NTFY_URL=https://ntfy.sh/your-topic \
./target/release/gluetun-monitor
```

## Configuration

All configuration is done via environment variables:

| Variable | Required | Default | Description |
| -------- | -------- | ------- | ----------- |
| `VPN_ALLOWED_ASNS` | Yes | - | Comma-separated list of allowed ASNs (e.g., `AS12345,AS67890`) |
| `GLUETUN_API_URL` | No | - | Gluetun API URL (e.g., `http://localhost:8000`) |
| `GLUETUN_API_KEY` | No | - | Gluetun API key if authentication is enabled |
| `NTFY_URL` | No | - | ntfy topic URL for notifications (e.g., `https://ntfy.sh/your-topic`) |
| `NTFY_INTERVAL_HOURS` | No | `2` | Hours between periodic status notifications (minimum: 1) |
| `VPN_CHECK_INTERVAL_MINUTES` | No | `5` | Minutes between VPN change detection checks (minimum: 1) |
| `RUST_LOG` | No | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

### Finding Your ASN

#### Option 1: Automated Discovery (Recommended)

Use the included script to automatically discover all ASNs used by your VPN provider:

```bash
# Set your gluetun-monitor host (default: localhost)
export GLUETUN_MONITOR_HOST=localhost  # or your server IP

# Run the discovery script
./scripts/discover-asns.sh
```

The script will:

1. Restart your docker-compose stack 20 times
2. Collect unique ASNs from each VPN connection
3. Output a ready-to-use `VPN_ALLOWED_ASNS` value

#### Option 2: Manual Lookup

1. Connect to your VPN
2. Visit [ifconfig.co/json](https://ifconfig.co/json)
3. Look for the `asn` field (e.g., `AS12345`)

## Uptime Kuma Integration

The `/status` and `/check` endpoints are designed for seamless integration with [Uptime Kuma](https://github.com/louislam/uptime-kuma) and other monitoring tools.

### Monitoring Setup

**HTTP(s) - Keyword Monitor:**

```
URL: http://gluetun-monitor:3010/status
Keyword: "configured":true
```

**HTTP(s) - Status Code Monitor:**

```
URL: http://gluetun-monitor:3010/check
Expected Status: 200
```

### Endpoint Behavior

- **`/status`**: Always returns `200 OK` with current VPN status and configuration
  - Use for **informational monitoring**
  - Check keyword `"configured":true` to verify setup
  - Returns full VPN details (IP, ASN, country, port forwarding)

- **`/check`**: Returns `200 OK` if VPN is healthy, `503 Service Unavailable` if not
  - Use for **health check monitoring**
  - Uptime Kuma will mark service as **DOWN** on 503
  - Perfect for alerting when VPN fails or ASN changes

### Example Uptime Kuma Configuration

1. **Add New Monitor** → HTTP(s)
2. **Friendly Name**: Gluetun VPN Health
3. **URL**: `http://gluetun-monitor:3010/check`
4. **Heartbeat Interval**: 60 seconds
5. **Retries**: 3
6. **Accepted Status Codes**: 200

Uptime Kuma will automatically alert you when:

- VPN connection fails
- ASN doesn't match allowed list
- Gluetun API becomes unavailable

## API Endpoints

### GET /status

Returns current VPN status and configuration.

**Response:**

```json
{
  "ip": "1.2.3.4",
  "asn": "AS12345",
  "org": "Your VPN Provider",
  "country": "Netherlands",
  "city": "Amsterdam",
  "region": "North Holland",
  "port_forwarded": 54321,
  "allowed_asns": ["AS12345", "AS67890"],
  "configured": true
}
```

### GET /check

Health check endpoint that returns HTTP 200 if VPN is healthy, 503 otherwise.

**Response (Healthy):**

```json
{
  "ok": true,
  "ip": "1.2.3.4",
  "asn": "AS12345",
  "org": "Your VPN Provider",
  "country": "Netherlands"
}
```

**Response (Unhealthy):**

```json
{
  "ok": false,
  "reason": "ASN not allowed",
  "ip": "5.6.7.8",
  "asn": "AS99999",
  "org": "Unknown Provider"
}
```

## Notifications

When configured with `NTFY_URL`, the monitor sends notifications for:

1. **Periodic Status Updates**: Regular health reports at configured intervals
2. **VPN Server Changes**: Immediate alerts when IP, country, or ASN changes

### Example Notification

```text
🔒 VPN Status Update

IP: 1.2.3.4
ASN: AS12345 (Your VPN Provider)
Location: Amsterdam, Netherlands
Port: 54321

✅ VPN is healthy
```

### Change Detection Notification

```text
⚠️ VPN Server Changed

IP: 1.2.3.4 → 5.6.7.8
Country: Netherlands → Germany
ASN: AS12345 → AS67890
```

## Architecture

```text
┌─────────────────┐
│  Gluetun        │
│  (VPN Client)   │
└────────┬────────┘
         │ API
         │
┌────────▼───────────────────────┐
│  Gluetun Monitor               │
│                                │
│  ┌─────────────────────────┐   │
│  │  IP Lookup              │   │
│  │  - Gluetun API          │   │
│  │  - ifconfig.co          │   │
│  │  - ip-api.com           │   │
│  └─────────────────────────┘   │
│                                │
│  ┌─────────────────────────┐   │
│  │  Monitoring             │   │
│  │  - Periodic Notifier    │   │
│  │  - Change Detector      │   │
│  └─────────────────────────┘   │
│                                │
│  ┌─────────────────────────┐   │
│  │  HTTP API               │   │
│  │  - /status              │   │
│  │  - /check               │   │
│  └─────────────────────────┘   │
└─────────────┬──────────────────┘
              │
              ▼
        ┌──────────┐
        │  ntfy    │
        └──────────┘
```

## Use Cases

- **Homelab Monitoring**: Ensure your self-hosted services are always behind a VPN
- **Privacy Verification**: Confirm your VPN is working and using expected servers
- **Automated Alerts**: Get notified immediately when VPN connection changes
- **Health Checks**: Integrate with monitoring systems via HTTP endpoints
- **Multi-VPN Setup**: Monitor multiple VPN connections with different ASN requirements

## Troubleshooting

### Monitor can't reach Gluetun API

Ensure the monitor is using the same network as Gluetun:

```yaml
network_mode: "service:gluetun"
```

### No notifications received

1. Verify `NTFY_URL` is set correctly
2. Check that the ntfy topic is accessible
3. Review logs: `docker logs gluetun-monitor`

### ASN validation failing

1. Find your current ASN: `curl ifconfig.co/json`
2. Update `VPN_ALLOWED_ASNS` with the correct ASN
3. Ensure ASN format includes "AS" prefix (e.g., `AS12345`)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Gluetun](https://github.com/qdm12/gluetun) - Excellent VPN client container
- [ntfy](https://ntfy.sh) - Simple notification service
- [Axum](https://github.com/tokio-rs/axum) - Ergonomic web framework
