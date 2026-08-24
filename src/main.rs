//! # Gluetun Monitor - Main Entry Point

use gluetun_monitor::{config, handlers, models, monitoring};

use axum::{routing::get, Router};
use models::AppState;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = config::Config::from_env();

    if let Some(ref url) = config.gluetun_url {
        info!("Using Gluetun API: {}", url);
        if config.gluetun_api_key.is_some() {
            info!("Gluetun API key configured");
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut allowed_asns_sorted: Vec<String> = config.allowed_asns.iter().cloned().collect();
    allowed_asns_sorted.sort();

    let state = AppState {
        allowed_asns: Arc::new(config.allowed_asns),
        allowed_asns_sorted: Arc::new(allowed_asns_sorted),
        allowed_providers: Arc::new(config.allowed_providers),
        home_ip: config.home_ip,
        client,
        ntfy_url: config.ntfy_url,
        gluetun_url: config.gluetun_url,
        gluetun_api_key: config.gluetun_api_key,
    };

    let notifier_state = state.clone();
    let interval_hours = config.notification_interval_hours;
    tokio::spawn(async move {
        monitoring::start_periodic_notifier(notifier_state, interval_hours).await;
    });

    let detector_state = state.clone();
    let check_interval = config.check_interval_minutes;
    tokio::spawn(async move {
        monitoring::start_change_detector(detector_state, check_interval).await;
    });

    let app = Router::new()
        .route("/status", get(handlers::status_handler))
        .route("/check", get(handlers::check_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3010")
        .await
        .map_err(|e| format!("Failed to bind to port 3010: {}", e))?;

    info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Server failed: {}", e))?;

    Ok(())
}
