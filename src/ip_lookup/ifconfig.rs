//! ifconfig.co IP Lookup
//!
//! Fast, simple JSON API for IP information.
//! Second fallback source after Gluetun API.

use crate::models::LookupResult;
use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
struct IfConfigResponse {
    ip: Option<String>,
    asn: Option<String>,
    asn_org: Option<String>,
    org: Option<String>,
    country: Option<String>,
}

/// Fetch IP information from ifconfig.co
///
/// # Parameters
/// - `client`: HTTP client for making requests
///
/// # Returns
/// - `Some(LookupResult)` with IP, ASN, org, and country on success
/// - `None` if request fails or response cannot be parsed
///
/// # Behavior
/// - Uses ifconfig.co's JSON API endpoint
/// - Ensures ASN has "AS" prefix (adds if missing)
/// - Uppercases ASN code
/// - Prefers `asn_org` field over `org` field
/// - Logs warnings on failure
///
/// # Example
/// ```text
/// let result = fetch_ifconfig(&client).await;
/// if let Some(info) = result {
///     println!("IP: {}, ASN: {}", info.ip?, info.asn?);
/// }
/// ```
pub async fn fetch_ifconfig(client: &Client) -> Option<LookupResult> {
    let resp = match client
        .get("https://ifconfig.co/json")
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("ifconfig.co API request failed: {}", e);
            return None;
        }
    };

    let data: IfConfigResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            warn!("Failed to parse ifconfig.co response: {}", e);
            return None;
        }
    };

    let ip = data.ip?;
    let asn = data.asn?;

    // Ensure ASN has "AS" prefix (convert to uppercase once)
    let asn_upper = asn.to_uppercase();
    let asn_formatted = if asn_upper.starts_with("AS") {
        asn_upper
    } else {
        format!("AS{}", asn)
    };

    let org = data.asn_org.or(data.org);

    Some(LookupResult {
        ip: Some(ip),
        asn: Some(asn_formatted),
        org,
        country: data.country,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    })
}
