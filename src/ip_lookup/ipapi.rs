//! ip-api.com IP Lookup
//!
//! Detailed geolocation and ASN information.
//! Third fallback source with comprehensive data.

use crate::models::LookupResult;
use reqwest::Client;
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
struct IpApiResponse {
    ip: Option<String>,
    asn: Option<serde_json::Value>, // Can be string or number
    org: Option<String>,
    organization: Option<String>,
    country_name: Option<String>,
}

/// Fetch IP information from ipapi.co
///
/// # Parameters
/// - `client`: HTTP client for making requests
///
/// # Returns
/// - `Some(LookupResult)` with IP, ASN, org, and country on success
/// - `None` if request fails or response cannot be parsed
///
/// # Behavior
/// - Uses ipapi.co's JSON API endpoint
/// - Handles ASN as either string or number in JSON response
/// - Uppercases ASN code
/// - Prefers `org` field over `organization` field
/// - Logs warnings on failure
///
/// # Note
/// Rate limited to ~1000 requests/day on free tier
///
/// # Example
/// ```text
/// let result = fetch_ipapi(&client).await;
/// if let Some(info) = result {
///     println!("IP: {}, ASN: {}", info.ip?, info.asn?);
/// }
/// ```
pub async fn fetch_ipapi(client: &Client) -> Option<LookupResult> {
    let resp = match client.get("https://ipapi.co/json/").send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("ipapi.co API request failed: {}", e);
            return None;
        }
    };

    let data: IpApiResponse = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            warn!("Failed to parse ipapi.co response: {}", e);
            return None;
        }
    };

    let ip = data.ip?;
    let asn_val = data.asn?;

    // Handle ASN as either string or number
    let asn_str = match asn_val {
        serde_json::Value::String(s) => s,
        serde_json::Value::Number(n) => n.to_string(),
        _ => return None,
    };

    let org = data.org.or(data.organization);

    Some(LookupResult {
        ip: Some(ip),
        asn: Some(asn_str.to_uppercase()),
        org,
        country: data.country_name,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    })
}
