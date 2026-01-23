//! Data Models
//!
//! Defines the core data structures used throughout the application.
//!
//! ## Key Types
//! - `LookupResult`: IP lookup response with ASN, location, and port forwarding info
//! - `StatusResponse`: Response for `/status` endpoint (informational)
//! - `CheckResponse`: Response for `/check` endpoint (health check)
//! - `AppState`: Shared application state passed to all handlers

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
    /// Check if the ASN is in the allowed set
    ///
    /// # Parameters
    /// - `allowed_asns`: Set of permitted ASN codes (e.g., {"AS12345", "AS67890"})
    ///
    /// # Returns
    /// - `true` if ASN is present and in the allowed set
    /// - `false` if ASN is None or not in the allowed set
    ///
    /// # Example
    /// ```rust,no_run
    /// # use std::collections::HashSet;
    /// # use gluetun_monitor::models::LookupResult;
    /// let mut allowed = HashSet::new();
    /// allowed.insert("AS12345".to_string());
    ///
    /// let result = LookupResult {
    ///     ip: None, asn: Some("AS12345".into()), org: None,
    ///     country: None, city: None, region: None,
    ///     port_forwarded: None, error: None
    /// };
    /// assert!(result.is_asn_allowed(&allowed));
    /// ```
    pub fn is_asn_allowed(&self, allowed_asns: &HashSet<String>) -> bool {
        self.asn
            .as_ref()
            .map(|asn| allowed_asns.contains(asn))
            .unwrap_or(false)
    }

    /// Format location as "City, Country" or fallback to just country or region
    ///
    /// # Returns
    /// Formatted location string based on available data:
    /// - `"City, Country"` if both present
    /// - `"Country"` if only country present
    /// - `"City"` if only city present
    /// - `"Region"` if only region present
    /// - `"Unknown"` if no location data
    ///
    /// # Example
    /// ```rust,no_run
    /// # use gluetun_monitor::models::LookupResult;
    /// let result = LookupResult {
    ///     ip: None, asn: None, org: None,
    ///     city: Some("Amsterdam".into()),
    ///     country: Some("Netherlands".into()),
    ///     region: None, port_forwarded: None, error: None
    /// };
    /// assert_eq!(result.format_location(), "Amsterdam, Netherlands");
    /// ```
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

// Custom serializer for Arc<Vec<String>> to serialize as array
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
    pub client: reqwest::Client,
    pub ntfy_url: Option<String>,
    pub gluetun_url: Option<String>,
    pub gluetun_api_key: Option<String>,
}
