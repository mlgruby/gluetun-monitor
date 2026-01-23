//! Health Check Handler
//!
//! Provides the `/check` endpoint for health monitoring.
//! Returns 200 OK if VPN is connected with allowed ASN, 503 otherwise.
//! Designed for Uptime Kuma and other monitoring tools.

use crate::{
    ip_lookup,
    models::{AppState, CheckResponse, LookupResult},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Error message when ASN validation is not configured
const ERR_ASNS_NOT_CONFIGURED: &str = "VPN_ALLOWED_ASNS not set";

/// Error message when connected ASN is not in allowed list
const ERR_ASN_NOT_ALLOWED: &str = "ASN not allowed";

/// Handler for `/check` health check endpoint
///
/// # Purpose
/// Validates VPN connection is active and using an allowed ASN.
/// Designed for monitoring tools like Uptime Kuma.
///
/// # Returns
/// - `200 OK` with `{"ok": true}` if VPN is connected with allowed ASN
/// - `503 Service Unavailable` with error details if:
///   - IP lookup fails
///   - ASN validation is not configured
///   - Connected ASN is not in allowed list
///
/// # Response Format
/// ```json
/// {
///   "ok": true/false,
///   "reason": "Error message (only if ok=false)",
///   "ip": "1.2.3.4",
///   "asn": "AS12345",
///   "country": "Netherlands"
/// }
/// ```
///
/// # HTTP Example
/// ```text
/// GET /check
/// 200 OK {"ok": true, "ip": "1.2.3.4", "asn": "AS12345"}
/// ```
pub async fn check_handler(State(state): State<AppState>) -> impl IntoResponse {
    let info = ip_lookup::lookup(
        &state.client,
        state.gluetun_url.as_deref(),
        state.gluetun_api_key.as_deref(),
    )
    .await;

    // Check for lookup errors
    if info.error.is_some() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CheckResponse {
                ok: false,
                reason: None, // Error is in the flattened lookup
                lookup: info,
            }),
        );
    }

    // Check if ASNs are configured
    if state.allowed_asns.is_empty() {
        let err_info = LookupResult {
            error: Some(ERR_ASNS_NOT_CONFIGURED.to_string()),
            ..info
        };
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CheckResponse {
                ok: false,
                reason: None,
                lookup: err_info,
            }),
        );
    }

    // Check if ASN is allowed
    if info.is_asn_allowed(&state.allowed_asns) {
        return (
            StatusCode::OK,
            Json(CheckResponse {
                ok: true,
                reason: None,
                lookup: info,
            }),
        );
    }

    // ASN not allowed
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(CheckResponse {
            ok: false,
            reason: Some(ERR_ASN_NOT_ALLOWED.to_string()),
            lookup: info,
        }),
    )
}
