//! VPN Change Detector
//!
//! Monitors VPN connection for changes (IP, ASN, location).
//! Sends notifications when changes are detected.
//! Runs continuously at configured check interval.

use crate::{ip_lookup, models::AppState, notification};
use tokio::time::Duration;
use tracing::{debug, info, warn};

/// Initial delay to wait for VPN connection to establish
///
/// **Why 35 seconds (vs 30 in periodic notifier)?**
/// - Change detector needs extra time to ensure baseline accuracy
/// - Gluetun needs ~30s to establish VPN connection
/// - Additional 5s buffer ensures first lookup captures stable state
/// - Prevents false "change" alerts from transient startup values
const VPN_INITIAL_DELAY_SECS: u64 = 35;

/// VPN state tracker for change detection
struct VpnState {
    ip: Option<String>,
    country: Option<String>,
    asn: Option<String>,
}

impl VpnState {
    fn new() -> Self {
        Self {
            ip: None,
            country: None,
            asn: None,
        }
    }

    /// Helper method to detect and report changes for a single field
    fn detect_field_change<T: PartialEq + Clone + std::fmt::Display>(
        stored: &mut Option<T>,
        current: &Option<T>,
        field_name: &str,
    ) -> Option<String> {
        match (stored.as_ref(), current.as_ref()) {
            (Some(prev), Some(curr)) if prev != curr => {
                let change = format!("{}: {} → {}", field_name, prev, curr);
                *stored = Some(curr.clone());
                Some(change)
            }
            (None, Some(curr)) => {
                *stored = Some(curr.clone());
                None
            }
            _ => None,
        }
    }

    /// Detect changes and return change details if any
    fn detect_changes(
        &mut self,
        current_ip: &Option<String>,
        current_country: &Option<String>,
        current_asn: &Option<String>,
    ) -> Option<String> {
        let mut changes = Vec::new();

        if let Some(change) = Self::detect_field_change(&mut self.ip, current_ip, "IP") {
            changes.push(change);
        }

        if let Some(change) =
            Self::detect_field_change(&mut self.country, current_country, "Country")
        {
            changes.push(change);
        }

        if let Some(change) = Self::detect_field_change(&mut self.asn, current_asn, "ASN") {
            changes.push(change);
        }

        if changes.is_empty() {
            None
        } else {
            Some(changes.join("\n"))
        }
    }
}

/// Start VPN change detection and notifications
///
/// # Purpose
/// Monitors VPN connection for changes (IP, ASN, country) and sends
/// notifications when changes are detected.
///
/// # Parameters
/// - `state`: Shared application state (config, HTTP client, etc.)
/// - `interval_minutes`: How often to check for changes (e.g., 5 = every 5 minutes)
///
/// # Behavior
/// 1. Waits 35 seconds for initial VPN connection
/// 2. Establishes baseline (first check, no notification)
/// 3. Checks every N minutes for changes
/// 4. Sends notification if IP, ASN, or country changes
/// 5. Runs forever until process terminates
///
/// # Requirements
/// - `NTFY_URL` must be configured (exits early if not set)
/// - At least one IP lookup source must be working
///
/// # Example
/// ```text
/// tokio::spawn(async move {
///     start_change_detector(state, 5).await; // Check every 5 minutes
/// });
/// ```
pub async fn start_change_detector(state: AppState, interval_minutes: u64) {
    // Check if NTFY_URL is configured (borrow check, no clone)
    if state.ntfy_url.is_none() {
        warn!("NTFY_URL not configured, change detection disabled");
        return;
    }
    let ntfy_url = state.ntfy_url.as_ref().unwrap();

    info!(
        "Starting change detector (checking every {} minutes)",
        interval_minutes
    );

    // Wait for initial VPN connection
    tokio::time::sleep(Duration::from_secs(VPN_INITIAL_DELAY_SECS)).await;

    let mut vpn_state = VpnState::new();
    let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60));

    // ========================================
    // BASELINE ESTABLISHMENT (First Check)
    // ========================================
    // Why this is needed: We call detect_changes() on the first lookup to establish
    // a baseline without sending a notification. This prevents false "change" alerts
    // when the service starts up.
    //
    // How it works:
    // 1. detect_changes() compares current values to stored state (initially all None)
    // 2. When stored state is None, it saves the current value without reporting a change
    // 3. Result: baseline is established silently, future calls will detect actual changes
    //
    // Example:
    //   First call:  detect_changes(None, "1.2.3.4") → stores "1.2.3.4", returns None
    //   Second call: detect_changes("1.2.3.4", "1.2.3.4") → no change, returns None
    //   Third call:  detect_changes("1.2.3.4", "5.6.7.8") → returns "IP: 1.2.3.4 → 5.6.7.8"
    let info = ip_lookup::lookup(
        &state.client,
        state.gluetun_url.as_deref(),
        state.gluetun_api_key.as_deref(),
    )
    .await;

    if info.error.is_none() {
        // Initialize baseline using detect_changes (returns None on first call)
        vpn_state.detect_changes(&info.ip, &info.country, &info.asn);
        info!(
            "Baseline established: IP={:?}, Country={:?}, ASN={:?}",
            vpn_state.ip, vpn_state.country, vpn_state.asn
        );
    }

    loop {
        interval.tick().await;

        debug!("Change detector: performing check");
        let info = ip_lookup::lookup(
            &state.client,
            state.gluetun_url.as_deref(),
            state.gluetun_api_key.as_deref(),
        )
        .await;

        if info.error.is_none() {
            if let Some(change_msg) = vpn_state.detect_changes(&info.ip, &info.country, &info.asn) {
                info!(
                    "VPN server change detected: {}",
                    change_msg.replace('\n', ", ")
                );

                // Send immediate notification about the change
                if let Err(e) = notification::send_notification(
                    &state.client,
                    ntfy_url,
                    &info,
                    &state.allowed_asns,
                    Some(&change_msg),
                )
                .await
                {
                    warn!("Failed to send change notification: {}", e);
                }
            }
        } else if let Some(err) = info.error {
            warn!("Change detector lookup failed: {}", err);
        }
    }
}
