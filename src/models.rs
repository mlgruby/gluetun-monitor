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

/// Response for /status endpoint
#[derive(Serialize)]
pub struct StatusResponse {
    #[serde(flatten)]
    pub lookup: LookupResult,
    pub allowed_asns: Vec<String>,
    pub configured: bool,
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
    pub client: reqwest::Client,
    pub ntfy_url: Option<String>,
    pub gluetun_url: Option<String>,
    pub gluetun_api_key: Option<String>,
}
