//! Gluetun IP Lookup
//!
//! Fetches IP information directly from the Gluetun API.
//! Primary lookup source when Gluetun API is configured.
//! Provides port forwarding information if available.

use crate::models::LookupResult;
use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
struct GluetunResponse {
    public_ip: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    organization: Option<String>,
}

#[derive(Deserialize)]
struct PortResponse {
    port: Option<u16>,
}

/// Fetch IP information from Gluetun API
///
/// # Parameters
/// - `client`: HTTP client for making requests
/// - `gluetun_url`: Base URL of Gluetun API (e.g., "http://gluetun:8000")
/// - `api_key`: Optional API key for authenticated requests
///
/// # Returns
/// - `Some(LookupResult)` with IP, ASN, location, and port forwarding info on success
/// - `None` if request fails or response cannot be parsed
///
/// # Behavior
/// 1. Fetches public IP info from Gluetun's `/v1/publicip/ip` endpoint
/// 2. Parses ASN from organization field (format: "AS12345 Provider Name")
/// 3. Attempts to fetch port forwarding info from `/v1/openvpn/portforwarded`
/// 4. Logs warnings on failure, returns None
///
/// # Example
/// ```text
/// let result = fetch_gluetun_ip(&client, "http://gluetun:8000", Some("key")).await;
/// if let Some(info) = result {
///     println!("IP: {}, ASN: {}", info.ip?, info.asn?);
/// }
/// ```
pub async fn fetch_gluetun_ip(
    client: &Client,
    gluetun_url: &str,
    api_key: Option<&str>,
) -> Option<LookupResult> {
    let url = format!("{}/v1/publicip/ip", gluetun_url);

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("Gluetun API request failed: {}", e);
            return None;
        }
    };

    let data: GluetunResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            warn!("Failed to parse Gluetun API response: {}", e);
            return None;
        }
    };

    let ip = data.public_ip?;

    // Extract ASN from organization field (format: "AS212238 Datacamp Limited")
    let (asn, org) = parse_organization(&data.organization);

    // Fetch port forwarding info
    let port_forwarded = fetch_port_forwarded(client, gluetun_url, api_key).await;

    Some(LookupResult {
        ip: Some(ip),
        asn,
        org,
        country: data.country,
        city: data.city,
        region: data.region,
        port_forwarded,
        error: None,
    })
}

/// Parse organization string to extract ASN and organization name
///
/// # Parameters
/// - `org_str`: Organization string from Gluetun API
///
/// # Returns
/// Tuple of `(asn, org_name)`:
/// - `asn`: ASN code (uppercased, e.g., "AS12345")
/// - `org_name`: Organization name (remaining text after first space)
///
/// # Format
/// Expects format: "AS12345 Provider Name" or "as12345 Provider Name"
/// - If no space found, treats entire string as org_name with no ASN
/// - ASN is always uppercased
///
/// # Examples
/// ```text
/// assert_eq!(
///     parse_organization(&Some("AS12345 Datacamp Limited".into())),
///     (Some("AS12345".into()), Some("Datacamp Limited".into()))
/// );
///
/// assert_eq!(
///     parse_organization(&Some("as99999 Test".into())),
///     (Some("AS99999".into()), Some("Test".into()))
/// );
///
/// assert_eq!(
///     parse_organization(&None),
///     (None, None)
/// );
/// ```
pub fn parse_organization(org_str: &Option<String>) -> (Option<String>, Option<String>) {
    let org_str = match org_str {
        Some(s) => s,
        None => return (None, None),
    };

    if let Some(asn_end) = org_str.find(' ') {
        let asn_part = &org_str[..asn_end];
        let org_part = org_str[asn_end + 1..].trim();
        (Some(asn_part.to_uppercase()), Some(org_part.to_string()))
    } else {
        (None, Some(org_str.clone()))
    }
}

/// Fetch port forwarding information from Gluetun
///
/// **Strategy:** Try with API key first (if provided), fallback to unauthenticated request.
/// Some Gluetun configurations allow unauthenticated port forwarding queries,
/// so we attempt both methods to maximize compatibility.
async fn fetch_port_forwarded(
    client: &Client,
    gluetun_url: &str,
    api_key: Option<&str>,
) -> Option<u16> {
    let url = format!("{}/v1/openvpn/portforwarded", gluetun_url);

    // Try with API key first if available
    if let Some(key) = api_key {
        if let Some(port) = try_fetch_port(&url, client, Some(key)).await {
            return Some(port);
        }
        warn!("Gluetun port forwarding with API key failed, trying without auth");
    }

    // Fallback: try without authentication
    try_fetch_port(&url, client, None).await
}

/// Helper to attempt port forwarding fetch (with or without API key)
async fn try_fetch_port(url: &str, client: &Client, api_key: Option<&str>) -> Option<u16> {
    let mut request = client.get(url);

    // Add API key header if provided
    if let Some(key) = api_key {
        request = request.header("X-API-Key", key);
    }

    match request.send().await {
        Ok(resp) => resp.json::<PortResponse>().await.ok()?.port,
        Err(e) => {
            let auth_msg = if api_key.is_some() {
                "with auth"
            } else {
                "without auth"
            };
            warn!("Gluetun port forwarding request {} failed: {}", auth_msg, e);
            None
        }
    }
}
