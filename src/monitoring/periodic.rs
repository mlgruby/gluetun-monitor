//! Periodic Notifier
//!
//! Sends periodic VPN status notifications at configured intervals.
//! Waits for VPN connection to establish before first notification.
//! Runs continuously in background.

use crate::{ip_lookup, models::AppState, notification};
use tokio::time::Duration;
use tracing::{error, info, warn};

const GLUETUN_STARTUP_DELAY_SECS: u64 = 30;

/// Start periodic health check notifications
pub async fn start_periodic_notifier(state: AppState, interval_hours: u64) {
    if state.ntfy_url.is_none() {
        warn!("NTFY_URL not configured, notifications disabled");
        return;
    }
    let ntfy_url = state.ntfy_url.as_ref().unwrap();

    info!(
        "Starting periodic notifier (every {} hours)",
        interval_hours
    );
    info!("Sending notifications to: {}", ntfy_url);

    // Wait for Gluetun to be ready before first notification
    if state.gluetun_url.is_some() {
        info!(
            "Waiting {} seconds for Gluetun to establish VPN connection",
            GLUETUN_STARTUP_DELAY_SECS
        );
        tokio::time::sleep(Duration::from_secs(GLUETUN_STARTUP_DELAY_SECS)).await;
    }

    let mut interval = tokio::time::interval(Duration::from_secs(interval_hours * 60 * 60));

    loop {
        interval.tick().await;

        let info = ip_lookup::lookup(
            &state.client,
            state.gluetun_url.as_deref(),
            state.gluetun_api_key.as_deref(),
        )
        .await;

        if let Err(e) = notification::send_notification(
            &state.client,
            ntfy_url,
            &info,
            &state.allowed_asns,
            &state.allowed_providers,
            state.home_ip.as_deref(),
            None,
        )
        .await
        {
            error!("Failed to send notification: {}", e);
        }
    }
}
