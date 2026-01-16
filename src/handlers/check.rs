//! Health Check Handler
//!
//! Provides the `/check` endpoint for health monitoring.
//! Returns 200 OK if VPN is connected with allowed ASN, 503 otherwise.
//! Designed for Uptime Kuma and other monitoring tools.

use crate::{
    ip_lookup,
    models::{AppState, CheckResponse},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Handler for /check endpoint
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
        let mut err_info = info;
        err_info.error = Some("VPN_ALLOWED_ASNS not set".to_string());
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
    if let Some(asn) = &info.asn {
        if state.allowed_asns.contains(asn) {
            return (
                StatusCode::OK,
                Json(CheckResponse {
                    ok: true,
                    reason: None,
                    lookup: info,
                }),
            );
        }
    }

    // ASN not allowed
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(CheckResponse {
            ok: false,
            reason: Some("ASN not allowed".to_string()),
            lookup: info,
        }),
    )
}
