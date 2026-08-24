//! Health Check Handler
//!
//! Provides the `/check` endpoint for health monitoring.
//! Returns 200 OK if VPN is connected and safe, 503 otherwise.

use crate::{
    ip_lookup,
    models::{AppState, CheckResponse},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

const ERR_VPN_LEAK: &str = "VPN leak detected: Public IP matches Home WAN IP";
const ERR_ASN_NOT_ALLOWED: &str = "ASN/Provider not allowed";

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
                reason: None,
                lookup: info,
            }),
        );
    }

    // Check for Home IP Leak
    if info.is_leak(state.home_ip.as_deref()) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(CheckResponse {
                ok: false,
                reason: Some(ERR_VPN_LEAK.to_string()),
                lookup: info,
            }),
        );
    }

    // Check if ASN or Provider is allowed
    if info.is_asn_allowed(&state.allowed_asns, &state.allowed_providers) {
        return (
            StatusCode::OK,
            Json(CheckResponse {
                ok: true,
                reason: None,
                lookup: info,
            }),
        );
    }

    // ASN / Provider not allowed
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(CheckResponse {
            ok: false,
            reason: Some(ERR_ASN_NOT_ALLOWED.to_string()),
            lookup: info,
        }),
    )
}
