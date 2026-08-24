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
        allowed_providers: Arc::new(HashSet::new()),
        home_ip: None,
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
        allowed_providers: Arc::new(HashSet::new()),
        home_ip: None,
        client: reqwest::Client::new(),
        ntfy_url: None,
        gluetun_url: None,
        gluetun_api_key: None,
    }
}

#[tokio::test]
async fn test_status_handler_executes() {
    let state = create_test_state();
    let _response = status_handler(axum::extract::State(state)).await;
}

#[tokio::test]
async fn test_status_handler_unconfigured_executes() {
    let state = create_unconfigured_state();
    let _response = status_handler(axum::extract::State(state)).await;
}

#[tokio::test]
async fn test_check_handler_no_asns_executes() {
    let state = create_unconfigured_state();
    let _response = check_handler(axum::extract::State(state)).await;
}

#[tokio::test]
async fn test_check_handler_with_configuration_executes() {
    let state = create_test_state();
    let _response = check_handler(axum::extract::State(state)).await;
}

#[test]
fn test_app_state_creation() {
    let mut allowed_asns = HashSet::new();
    allowed_asns.insert("AS12345".to_string());

    let mut allowed_asns_sorted: Vec<String> = allowed_asns.iter().cloned().collect();
    allowed_asns_sorted.sort();

    let state = AppState {
        allowed_asns: Arc::new(allowed_asns),
        allowed_asns_sorted: Arc::new(allowed_asns_sorted),
        allowed_providers: Arc::new(HashSet::new()),
        home_ip: None,
        client: reqwest::Client::new(),
        ntfy_url: Some("https://ntfy.sh/test".to_string()),
        gluetun_url: Some("http://localhost:8000".to_string()),
        gluetun_api_key: Some("test-key".to_string()),
    };

    assert_eq!(state.allowed_asns.len(), 1);
    assert!(state.ntfy_url.is_some());
    assert!(state.gluetun_url.is_some());
}
