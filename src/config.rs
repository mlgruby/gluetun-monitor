// Configuration Module
//!
//! Handles loading and parsing of application configuration from environment variables.
//!
//! ## Environment Variables
//! - `VPN_ALLOWED_ASNS`: Comma-separated list of allowed ASNs (required)
//! - `GLUETUN_API_URL`: Gluetun API endpoint (optional)
//! - `GLUETUN_API_KEY`: Gluetun API key (optional)
//! - `NTFY_URL`: ntfy.sh notification URL (optional)
//! - `NTFY_INTERVAL_HOURS`: Notification interval in hours (default: 2, min: 1)
//! - `VPN_CHECK_INTERVAL_MINUTES`: VPN check interval in minutes (default: 5, min: 1)

use std::{collections::HashSet, env};

/// Application configuration loaded from environment variables
pub struct Config {
    pub allowed_asns: HashSet<String>,
    pub ntfy_url: Option<String>,
    pub gluetun_url: Option<String>,
    pub gluetun_api_key: Option<String>,
    pub notification_interval_hours: u64,
    pub check_interval_minutes: u64,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let allowed_env = env::var("VPN_ALLOWED_ASNS").unwrap_or_default();
        let allowed_asns: HashSet<String> = allowed_env
            .split(',')
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .collect();

        let ntfy_url = env::var("NTFY_URL").ok();
        let gluetun_url = env::var("GLUETUN_API_URL").ok();
        let gluetun_api_key = env::var("GLUETUN_API_KEY").ok();

        // Parse notification interval, default to 2 hours, minimum 1 hour
        let notification_interval_hours = env::var("NTFY_INTERVAL_HOURS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2)
            .max(1); // Ensure at least 1 hour

        // Parse check interval, default to 5 minutes, minimum 1 minute
        let check_interval_minutes = env::var("VPN_CHECK_INTERVAL_MINUTES")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(5)
            .max(1); // Ensure at least 1 minute

        Self {
            allowed_asns,
            ntfy_url,
            gluetun_url,
            gluetun_api_key,
            notification_interval_hours,
            check_interval_minutes,
        }
    }
}
