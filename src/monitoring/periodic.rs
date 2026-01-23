//! Periodic Notifier
//!
//! Sends periodic VPN status notifications at configured intervals.
//! Waits for VPN connection to establish before first notification.
//! Runs continuously in background.

use crate::{ip_lookup, models::AppState, notification};
use tokio::time::Duration;
use tracing::{error, info, warn};

/// Startup delay to wait for Gluetun to establish VPN connection
///
/// **Why 30 seconds?**
/// - Typical time for Gluetun to connect to VPN server
/// - Ensures first notification has valid IP/ASN data
/// - Shorter than change detector (35s) since notifications are informational only
const GLUETUN_STARTUP_DELAY_SECS: u64 = 30;

/// Start periodic health check notifications
///
/// # Purpose
/// Sends regular VPN status notifications at configured intervals.
/// Useful for confirming service is alive and VPN is working.
///
/// # Parameters
/// - `state`: Shared application state (config, HTTP client, etc.)
/// - `interval_hours`: How often to send notifications (e.g., 2 = every 2 hours)
///
/// # Behavior
/// 1. Waits 30 seconds for Gluetun to establish VPN (if configured)
/// 2. Sends notification immediately after delay
/// 3. Sends subsequent notifications every N hours
/// 4. Runs forever until process terminates
///
/// # Requirements
/// - `NTFY_URL` must be configured (exits early if not set)
/// - At least one IP lookup source must be working
///
/// # Example
/// ```text
/// tokio::spawn(async move {
///     start_periodic_notifier(state, 2).await; // Notify every 2 hours
/// });
/// ```
pub async fn start_periodic_notifier(state: AppState, interval_hours: u64) {
    // Check if NTFY_URL is configured (borrow check, no clone)
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
            None,
        )
        .await
        {
            error!("Failed to send notification: {}", e);
        }
    }
}
