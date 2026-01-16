//! ifconfig.co IP Lookup
//!
//! Fast, simple JSON API for IP information.
//! Second fallback source after Gluetun API.

use crate::models::LookupResult;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct IfConfigResponse {
    ip: Option<String>,
    asn: Option<String>,
    asn_org: Option<String>,
    org: Option<String>,
    country: Option<String>,
}

/// Fetch IP information from ifconfig.co
pub async fn fetch_ifconfig(client: &Client) -> Option<LookupResult> {
    let resp = client
        .get("https://ifconfig.co/json")
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;

    let data: IfConfigResponse = resp.json().await.ok()?;

    let ip = data.ip?;
    let asn = data.asn?;

    // Ensure ASN has "AS" prefix
    let asn_formatted = if asn.to_uppercase().starts_with("AS") {
        asn.to_uppercase()
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
