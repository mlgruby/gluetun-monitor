# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-15

### Added

- Initial release
- VPN health monitoring via Gluetun API integration
- ASN validation against allowed list
- Multiple IP lookup providers (Gluetun, ifconfig.co, ip-api.com)
- Port forwarding status monitoring
- Change detection for IP, country, and ASN
- Periodic status notifications via ntfy
- Immediate notifications on VPN server changes
- HTTP API endpoints (`/status` and `/check`)
- Configurable check intervals and notification frequency
- Docker support with multi-stage builds
- Comprehensive logging with configurable levels
- Comprehensive test suite with 20 tests across 4 test files
- ASN discovery helper script (`scripts/discover-asns.sh`)
- File-level documentation for all 16 Rust source files
- Health check endpoint in Dockerfile
- Non-root user in Docker container for security
- Optimized Dockerfile with binary stripping and minimal runtime (20.8MB)

[Unreleased]: https://github.com/yourusername/gluetun-monitor/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/gluetun-monitor/releases/tag/v0.1.0
