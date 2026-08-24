//! Data Models
//!
//! Defines the core data structures used throughout the application.

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

/// Result from IP lookup services
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LookupResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_forwarded: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl LookupResult {
    /// Check if the connection matches permitted ASNs, allowed provider keywords, or unrestricted mode
    pub fn is_asn_allowed(
        &self,
        allowed_asns: &HashSet<String>,
        allowed_providers: &HashSet<String>,
    ) -> bool {
        if let Some(asn) = self.asn.as_ref() {
            if allowed_asns.contains(asn) {
                return true;
            }
        }

        if let Some(org) = self.org.as_ref() {
            let upper_org = org.to_uppercase();
            for provider in allowed_providers {
                if upper_org.contains(provider) {
                    return true;
                }
            }
        }

        if allowed_asns.is_empty() && allowed_providers.is_empty() {
            return true;
        }

        if allowed_asns.is_empty() {
            return true;
        }

        false
    }

    /// Check if current IP leaked and matches Home WAN IP
    pub fn is_leak(&self, home_ip: Option<&str>) -> bool {
        match (self.ip.as_deref(), home_ip) {
            (Some(curr_ip), Some(h_ip)) if !h_ip.is_empty() => curr_ip.trim() == h_ip.trim(),
            _ => false,
        }
    }

    /// Format location as "City, Country" or fallback
    pub fn format_location(&self) -> String {
        match (&self.city, &self.country) {
            (Some(city), Some(country)) => format!("{}, {}", city, country),
            (None, Some(country)) => country.to_string(),
            (Some(city), None) => city.to_string(),
            (None, None) => self.region.as_deref().unwrap_or("Unknown").to_string(),
        }
    }
}

/// Response for /status endpoint
#[derive(Serialize)]
pub struct StatusResponse {
    #[serde(flatten)]
    pub lookup: LookupResult,
    #[serde(serialize_with = "serialize_arc_vec")]
    pub allowed_asns: Arc<Vec<String>>,
    pub configured: bool,
}

fn serialize_arc_vec<S>(arc: &Arc<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    arc.as_ref().serialize(serializer)
}

/// Response for /check endpoint
#[derive(Serialize)]
pub struct CheckResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(flatten)]
    pub lookup: LookupResult,
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub allowed_asns: Arc<HashSet<String>>,
    pub allowed_asns_sorted: Arc<Vec<String>>,
    pub allowed_providers: Arc<HashSet<String>>,
    pub home_ip: Option<String>,
    pub client: reqwest::Client,
    pub ntfy_url: Option<String>,
    pub gluetun_url: Option<String>,
    pub gluetun_api_key: Option<String>,
}
