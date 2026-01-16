//! Gluetun IP Lookup
//!
//! Fetches IP information directly from the Gluetun API.
//! Primary lookup source when Gluetun API is configured.
//! Provides port forwarding information if available.

use crate::models::LookupResult;
use reqwest::Client;
use serde::Deserialize;

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
pub async fn fetch_gluetun_ip(
    client: &Client,
    gluetun_url: &str,
    api_key: Option<&str>,
) -> Option<LookupResult> {
    let url = format!("{}/v1/publicip/ip", gluetun_url);

    let resp = client.get(&url).send().await.ok()?;
    let data: GluetunResponse = resp.json().await.ok()?;

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

/// Parse organization string to extract ASN and org name
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
async fn fetch_port_forwarded(
    client: &Client,
    gluetun_url: &str,
    api_key: Option<&str>,
) -> Option<u16> {
    let url = format!("{}/v1/openvpn/portforwarded", gluetun_url);

    // Try with API key first if available
    if let Some(key) = api_key {
        if let Ok(resp) = client.get(&url).header("X-API-Key", key).send().await {
            if let Ok(data) = resp.json::<PortResponse>().await {
                return data.port;
            }
        }
    }

    // Fallback: try without auth
    if let Ok(resp) = client.get(&url).send().await {
        if let Ok(data) = resp.json::<PortResponse>().await {
            return data.port;
        }
    }

    None
}
