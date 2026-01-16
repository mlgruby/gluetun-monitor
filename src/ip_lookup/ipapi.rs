//! ip-api.com IP Lookup
//!
//! Detailed geolocation and ASN information.
//! Third fallback source with comprehensive data.

use crate::models::LookupResult;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct IpApiResponse {
    ip: Option<String>,
    asn: Option<serde_json::Value>, // Can be string or number
    org: Option<String>,
    organization: Option<String>,
    country_name: Option<String>,
}

/// Fetch IP information from ipapi.co
pub async fn fetch_ipapi(client: &Client) -> Option<LookupResult> {
    let resp = client.get("https://ipapi.co/json/").send().await.ok()?;

    let data: IpApiResponse = resp.json().await.ok()?;

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
