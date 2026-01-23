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
use tracing::{debug, error};

/// Error message when all IP lookup services fail
const ERR_ALL_LOOKUPS_FAILED: &str = "All IP lookup services failed";

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
        debug!("Attempting IP lookup via Gluetun API");
        if let Some(res) = fetch_gluetun_ip(client, url, api_key).await {
            debug!("IP lookup successful via Gluetun API");
            return res;
        }
        debug!("Gluetun API lookup failed, falling back to external services");
    }

    // Fallback to ifconfig.co
    debug!("Attempting IP lookup via ifconfig.co");
    if let Some(res) = fetch_ifconfig(client).await {
        debug!("IP lookup successful via ifconfig.co");
        return res;
    }
    debug!("ifconfig.co lookup failed, trying ip-api.com");

    // Fallback to ip-api.com
    debug!("Attempting IP lookup via ip-api.com");
    if let Some(res) = fetch_ipapi(client).await {
        debug!("IP lookup successful via ip-api.com");
        return res;
    }

    // All lookups failed
    error!("All IP lookup services failed (Gluetun, ifconfig.co, ip-api.com)");
    LookupResult {
        ip: None,
        asn: None,
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: Some(ERR_ALL_LOOKUPS_FAILED.to_string()),
    }
}
