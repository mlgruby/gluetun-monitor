// Handler integration tests
//
// These tests verify the HTTP handlers work correctly by calling them directly.
// Since handlers return `impl IntoResponse`, we test that they execute without panicking.

use gluetun_monitor::{
    handlers::{check_handler, status_handler},
    models::AppState,
};
use std::{collections::HashSet, sync::Arc};

fn create_test_state() -> AppState {
    let mut allowed_asns = HashSet::new();
    allowed_asns.insert("AS12345".to_string());
    allowed_asns.insert("AS67890".to_string());

    let mut allowed_asns_sorted: Vec<String> = allowed_asns.iter().cloned().collect();
    allowed_asns_sorted.sort();

    AppState {
        allowed_asns: Arc::new(allowed_asns),
        allowed_asns_sorted: Arc::new(allowed_asns_sorted),
        client: reqwest::Client::new(),
        ntfy_url: Some("https://ntfy.sh/test".to_string()),
        gluetun_url: None,
        gluetun_api_key: None,
    }
}

fn create_unconfigured_state() -> AppState {
    AppState {
        allowed_asns: Arc::new(HashSet::new()),
        allowed_asns_sorted: Arc::new(Vec::new()),
        client: reqwest::Client::new(),
        ntfy_url: None,
        gluetun_url: None,
        gluetun_api_key: None,
    }
}

#[tokio::test]
async fn test_status_handler_executes() {
    // Given: A configured application state
    let state = create_test_state();

    // When: The status handler is called
    let _response = status_handler(axum::extract::State(state)).await;

    // Then: Handler executes without panicking
    // (If we reach here, the handler executed successfully)
}

#[tokio::test]
async fn test_status_handler_unconfigured_executes() {
    // Given: An unconfigured application state (no allowed ASNs)
    let state = create_unconfigured_state();

    // When: The status handler is called
    let _response = status_handler(axum::extract::State(state)).await;

    // Then: Handler executes without panicking even when unconfigured
}

#[tokio::test]
async fn test_check_handler_no_asns_executes() {
    // Given: An unconfigured state with no allowed ASNs
    let state = create_unconfigured_state();

    // When: The check handler is called
    let _response = check_handler(axum::extract::State(state)).await;

    // Then: Handler executes and returns proper error response
    // (No panic occurs even though ASNs are not configured)
}

#[tokio::test]
async fn test_check_handler_with_configuration_executes() {
    // Given: A configured state with allowed ASNs
    let state = create_test_state();

    // When: The check handler is called
    let _response = check_handler(axum::extract::State(state)).await;

    // Then: Handler executes without panicking
    // (IP lookup will fail but handler should not panic)
}

#[test]
fn test_app_state_creation() {
    // Given: Valid configuration values
    let mut allowed_asns = HashSet::new();
    allowed_asns.insert("AS12345".to_string());

    let mut allowed_asns_sorted: Vec<String> = allowed_asns.iter().cloned().collect();
    allowed_asns_sorted.sort();

    // When: Creating an AppState
    let state = AppState {
        allowed_asns: Arc::new(allowed_asns),
        allowed_asns_sorted: Arc::new(allowed_asns_sorted),
        client: reqwest::Client::new(),
        ntfy_url: Some("https://ntfy.sh/test".to_string()),
        gluetun_url: Some("http://localhost:8000".to_string()),
        gluetun_api_key: Some("test-key".to_string()),
    };

    // Then: State should be properly initialized with all fields
    assert_eq!(state.allowed_asns.len(), 1);
    assert!(state.ntfy_url.is_some());
    assert!(state.gluetun_url.is_some());
}
