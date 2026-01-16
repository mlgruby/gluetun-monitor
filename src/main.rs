//! # Gluetun Monitor - Main Entry Point
//!
//! This is the main entry point for the VPN health monitoring service.
//! It initializes the HTTP server, spawns background monitoring tasks,
//! and sets up the API endpoints for health checking and status reporting.
//!
//! ## Responsibilities
//! - Initialize tracing/logging
//! - Load configuration from environment variables
//! - Create HTTP client and application state
//! - Spawn periodic notification task
//! - Spawn VPN change detection task
//! - Start Axum HTTP server on port 3010

use gluetun_monitor::{config, handlers, models, monitoring};

use axum::{routing::get, Router};
use models::AppState;
use std::sync::Arc;
use tokio::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    // Log Gluetun configuration
    if let Some(ref url) = config.gluetun_url {
        info!("Using Gluetun API: {}", url);
        if config.gluetun_api_key.is_some() {
            info!("Gluetun API key configured");
        }
    }

    // Create HTTP client
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Create application state
    let state = AppState {
        allowed_asns: Arc::new(config.allowed_asns),
        client,
        ntfy_url: config.ntfy_url,
        gluetun_url: config.gluetun_url,
        gluetun_api_key: config.gluetun_api_key,
    };

    // Spawn periodic notifier in background
    let notifier_state = state.clone();
    let interval_hours = config.notification_interval_hours;
    tokio::spawn(async move {
        monitoring::start_periodic_notifier(notifier_state, interval_hours).await;
    });

    // Spawn change detector in background
    let detector_state = state.clone();
    let check_interval = config.check_interval_minutes;
    tokio::spawn(async move {
        monitoring::start_change_detector(detector_state, check_interval).await;
    });

    // Create router
    let app = Router::new()
        .route("/status", get(handlers::status_handler))
        .route("/check", get(handlers::check_handler))
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3010")
        .await
        .expect("Failed to bind to port 3010");

    info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.expect("Server failed");
}
