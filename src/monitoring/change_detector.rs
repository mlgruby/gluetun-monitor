//! VPN Change Detector
//!
//! Monitors VPN connection for changes (IP, ASN, location).
//! Sends notifications when changes are detected.
//! Runs continuously at configured check interval.

use crate::{ip_lookup, models::AppState, notification};
use tokio::time::Duration;
use tracing::{debug, info, warn};

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

    /// Detect changes and return change details if any
    fn detect_changes(
        &mut self,
        current_ip: &Option<String>,
        current_country: &Option<String>,
        current_asn: &Option<String>,
    ) -> Option<String> {
        let mut changes = Vec::new();

        // Check IP change - only clone if changed
        if let Some(ip) = current_ip {
            if let Some(ref prev_ip) = self.ip {
                if ip != prev_ip {
                    changes.push(format!("IP: {} → {}", prev_ip, ip));
                    self.ip = Some(ip.clone());
                }
            } else {
                self.ip = Some(ip.clone());
            }
        }

        // Check country change - only clone if changed
        if let Some(country) = current_country {
            if let Some(ref prev_country) = self.country {
                if country != prev_country {
                    changes.push(format!("Country: {} → {}", prev_country, country));
                    self.country = Some(country.clone());
                }
            } else {
                self.country = Some(country.clone());
            }
        }

        // Check ASN change - only clone if changed
        if let Some(asn) = current_asn {
            if let Some(ref prev_asn) = self.asn {
                if asn != prev_asn {
                    changes.push(format!("ASN: {} → {}", prev_asn, asn));
                    self.asn = Some(asn.clone());
                }
            } else {
                self.asn = Some(asn.clone());
            }
        }

        if changes.is_empty() {
            None
        } else {
            Some(changes.join("\n"))
        }
    }
}

/// Start VPN change detection and notifications
pub async fn start_change_detector(state: AppState, interval_minutes: u64) {
    let ntfy_url = match &state.ntfy_url {
        Some(url) => url.clone(),
        None => {
            warn!("NTFY_URL not configured, change detection disabled");
            return;
        }
    };

    info!(
        "Starting change detector (checking every {} minutes)",
        interval_minutes
    );

    // Wait for initial VPN connection
    tokio::time::sleep(Duration::from_secs(35)).await;

    let mut vpn_state = VpnState::new();
    let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60)); // 5 minutes

    // First check: establish baseline (don't send notification)
    let info = ip_lookup::lookup(
        &state.client,
        state.gluetun_url.as_deref(),
        state.gluetun_api_key.as_deref(),
    )
    .await;

    if info.error.is_none() {
        // Initialize baseline using detect_changes
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
                    &ntfy_url,
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
