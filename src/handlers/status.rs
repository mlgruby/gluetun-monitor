//! Status Handler
//!
//! Provides the `/status` endpoint for informational monitoring.
//! Always returns 200 OK with current VPN status and configuration.

use crate::{
    ip_lookup,
    models::{AppState, StatusResponse},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// Handler for `/status` informational endpoint
///
/// # Purpose
/// Returns current VPN connection details and configuration status.
/// Always returns 200 OK (even on errors) for monitoring visibility.
///
/// # Returns
/// Always returns `200 OK` with VPN status information
///
/// # Response Format
/// ```json
/// {
///   "ip": "1.2.3.4",
///   "asn": "AS12345",
///   "country": "Netherlands",
///   "city": "Amsterdam",
///   "org": "Provider Name",
///   "port_forwarded": 12345,
///   "allowed_asns": ["AS12345", "AS67890"],
///   "configured": true,
///   "error": "Error message (only if lookup failed)"
/// }
/// ```
///
/// # HTTP Example
/// ```text
/// GET /status
/// 200 OK {"ip": "1.2.3.4", "asn": "AS12345", "configured": true, ...}
/// ```
pub async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let lookup = ip_lookup::lookup(
        &state.client,
        state.gluetun_url.as_deref(),
        state.gluetun_api_key.as_deref(),
    )
    .await;

    // Clone Arc pointer (cheap - just increments ref count, no Vec clone!)
    let configured = !state.allowed_asns.is_empty();

    let response = StatusResponse {
        lookup,
        allowed_asns: Arc::clone(&state.allowed_asns_sorted),
        configured,
    };

    (StatusCode::OK, Json(response))
}
