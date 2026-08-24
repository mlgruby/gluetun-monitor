# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1](https://github.com/mlgruby/gluetun-monitor/compare/gluetun-monitor-v0.2.0...gluetun-monitor-v0.2.1) (2026-08-24)


### Features

* initial release with comprehensive monitoring and testing ([c3a99bc](https://github.com/mlgruby/gluetun-monitor/commit/c3a99bc0262e82c1c36781b667f2553592632aac))
* initial release with comprehensive monitoring and testing ([ab7d6ef](https://github.com/mlgruby/gluetun-monitor/commit/ab7d6ef545fc813b45fe4ce35ca618b3f6e7610d))
* smart provider matching and home IP leak detection ([d4e90f6](https://github.com/mlgruby/gluetun-monitor/commit/d4e90f63325845596362516cb838fb8f90acae82))
* smart provider matching, zero-maintenance ASN mode, and home IP leak detection ([e4f1ddf](https://github.com/mlgruby/gluetun-monitor/commit/e4f1ddf858ff30a288494ae8a59d16394a3cef28))


### Bug Fixes

* **ci:** fix docker metadata tag format for release workflow ([e7928a4](https://github.com/mlgruby/gluetun-monitor/commit/e7928a457215f3a3afc6b2444809b6cd0ed2bbea))
* **ci:** fix docker metadata tag format for release workflow ([49e1e76](https://github.com/mlgruby/gluetun-monitor/commit/49e1e76ffef19cf4dccb7ae898c6661fdd8179ac))
* update cargo packages ([8a06ace](https://github.com/mlgruby/gluetun-monitor/commit/8a06ace515d084307490ae87c29b7108ac26ed0e))
* update docker ghcr.io url ([756906c](https://github.com/mlgruby/gluetun-monitor/commit/756906cefa848c59672565ea2982228ef8c9ad22))

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
