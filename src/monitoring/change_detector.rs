//! VPN Change Detector
//!
//! Monitors VPN connection for changes (IP, ASN, location).
//! Sends notifications when changes are detected.
//! Runs continuously at configured check interval.

use crate::{ip_lookup, models::AppState, notification};
use tokio::time::Duration;
use tracing::{debug, info, warn};

const VPN_INITIAL_DELAY_SECS: u64 = 35;

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

pub async fn start_change_detector(state: AppState, interval_minutes: u64) {
    if state.ntfy_url.is_none() {
        warn!("NTFY_URL not configured, change detection disabled");
        return;
    }
    let ntfy_url = state.ntfy_url.as_ref().unwrap();

    info!(
        "Starting change detector (checking every {} minutes)",
        interval_minutes
    );

    tokio::time::sleep(Duration::from_secs(VPN_INITIAL_DELAY_SECS)).await;

    let mut vpn_state = VpnState::new();
    let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60));

    let info = ip_lookup::lookup(
        &state.client,
        state.gluetun_url.as_deref(),
        state.gluetun_api_key.as_deref(),
    )
    .await;

    if info.error.is_none() {
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

                if let Err(e) = notification::send_notification(
                    &state.client,
                    ntfy_url,
                    &info,
                    &state.allowed_asns,
                    &state.allowed_providers,
                    state.home_ip.as_deref(),
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
