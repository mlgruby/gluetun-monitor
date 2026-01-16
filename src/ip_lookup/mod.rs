//! IP Lookup Module
//!
//! Handles fetching the current public IP address and related information (ASN, country, etc.)
//! from multiple sources with fallback support.
//!
//! ## Lookup Sources (in order)
//! 1. Gluetun API (if configured) - Primary source with port forwarding info
//! 2. ifconfig.co - Fast, simple JSON API
//! 3. ip-api.com - Detailed geolocation and ASN information

pub mod gluetun; // Public for testing
mod ifconfig;
mod ipapi;

use crate::models::LookupResult;
use reqwest::Client;
use tracing::error;

pub use gluetun::fetch_gluetun_ip;
pub use ifconfig::fetch_ifconfig;
pub use ipapi::fetch_ipapi;

/// Perform IP lookup with fallback strategy
///
/// Tries Gluetun API first (if configured), then falls back to external services
pub async fn lookup(
    client: &Client,
    gluetun_url: Option<&str>,
    api_key: Option<&str>,
) -> LookupResult {
    // Try Gluetun API first if available
    if let Some(url) = gluetun_url {
        if let Some(res) = fetch_gluetun_ip(client, url, api_key).await {
            return res;
        }
    }

    // Fallback to external services
    if let Some(res) = fetch_ifconfig(client).await {
        return res;
    }

    if let Some(res) = fetch_ipapi(client).await {
        return res;
    }

    // All lookups failed
    error!("All IP lookup services failed");
    LookupResult {
        ip: None,
        asn: None,
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: Some("ASN lookup failed".to_string()),
    }
}
