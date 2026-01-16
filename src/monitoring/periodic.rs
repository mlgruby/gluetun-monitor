//! Periodic Notifier
//!
//! Sends periodic VPN status notifications at configured intervals.
//! Waits for VPN connection to establish before first notification.
//! Runs continuously in background.

use crate::{ip_lookup, models::AppState, notification};
use tokio::time::Duration;
use tracing::{error, info, warn};

/// Start periodic health check notifications
pub async fn start_periodic_notifier(state: AppState, interval_hours: u64) {
    let ntfy_url = match &state.ntfy_url {
        Some(url) => url.clone(),
        None => {
            warn!("NTFY_URL not configured, notifications disabled");
            return;
        }
    };

    info!(
        "Starting periodic notifier (every {} hours)",
        interval_hours
    );
    info!("Sending notifications to: {}", ntfy_url);

    // Wait for Gluetun to be ready before first notification
    if state.gluetun_url.is_some() {
        info!("Waiting 30 seconds for Gluetun to establish VPN connection");
        tokio::time::sleep(Duration::from_secs(30)).await;
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
            &ntfy_url,
            &info,
            &state.allowed_asns,
            None,
        )
        .await
        {
            error!("Failed to send notification: {}", e);
        }
    }
}
