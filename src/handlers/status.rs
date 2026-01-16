//! Status Handler
//!
//! Provides the `/status` endpoint for informational monitoring.
//! Always returns 200 OK with current VPN status and configuration.

use crate::{
    ip_lookup,
    models::{AppState, StatusResponse},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// Handler for /status endpoint
pub async fn status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let lookup = ip_lookup::lookup(
        &state.client,
        state.gluetun_url.as_deref(),
        state.gluetun_api_key.as_deref(),
    )
    .await;

    let mut allowed_vec: Vec<String> = state.allowed_asns.iter().cloned().collect();
    allowed_vec.sort();

    let configured = !state.allowed_asns.is_empty();

    let response = StatusResponse {
        lookup,
        allowed_asns: allowed_vec,
        configured,
    };

    (StatusCode::OK, Json(response))
}
