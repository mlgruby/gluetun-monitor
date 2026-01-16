//! ntfy.sh Notification Service
//!
//! Sends rich notifications to ntfy.sh with VPN status information.
//! Includes formatted messages with emojis, priority levels, and tags.
//! Supports both periodic updates and change notifications.

use crate::models::LookupResult;
use reqwest::Client;
use std::collections::HashSet;
use tokio::time::Duration;
use tracing::{error, info, warn};

/// Send notification via ntfy with retry logic
pub async fn send_notification(
    client: &Client,
    ntfy_url: &str,
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    change_details: Option<&str>,
) -> Result<(), String> {
    let message = build_message(info, allowed_asns, change_details);
    let title = determine_title(info, allowed_asns, change_details);
    let priority = determine_priority(info, allowed_asns, change_details);

    // Retry logic: 3 attempts with exponential backoff
    for attempt in 1..=3 {
        let request = client
            .post(ntfy_url)
            .header("Title", title)
            .header("Priority", priority)
            .header("Tags", "vpn,network")
            .body(message.clone());

        match request.send().await {
            Ok(resp) if resp.status().is_success() => {
                info!("Notification sent successfully");
                return Ok(());
            }
            Ok(resp) => {
                let err = format!("Notification failed with status: {}", resp.status());
                if attempt < 3 {
                    warn!("{}, retrying ({}/3)", err, attempt);
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt as u32))).await;
                } else {
                    error!("{}", err);
                    return Err(err);
                }
            }
            Err(e) => {
                let err = format!("Failed to send notification: {}", e);
                if attempt < 3 {
                    warn!("{}, retrying ({}/3)", err, attempt);
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt as u32))).await;
                } else {
                    error!("{}", err);
                    return Err(err);
                }
            }
        }
    }

    Err("All retry attempts failed".to_string())
}

/// Build notification message with status and details
fn build_message(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    change_details: Option<&str>,
) -> String {
    let is_allowed = info
        .asn
        .as_ref()
        .is_some_and(|asn| allowed_asns.contains(asn));
    let status_emoji = if is_allowed { "✅" } else { "⚠️" };
    let status_text = if is_allowed { "Allowed" } else { "Not Allowed" };
    let proton_badge = if is_allowed {
        "🔒 Proton VPN"
    } else {
        "⚡ Unknown Provider"
    };

    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let org_info = info
        .org
        .as_ref()
        .map(|org| format!("🏢 Provider: {}\n", org))
        .unwrap_or_default();

    // Build location string with city and region if available
    let location = match (&info.city, &info.region, &info.country) {
        (Some(city), Some(region), Some(country)) => format!("{}, {} ({})", city, region, country),
        (Some(city), None, Some(country)) => format!("{}, {}", city, country),
        (None, None, Some(country)) => country.to_string(),
        _ => "Unknown".to_string(),
    };

    let port_info = info
        .port_forwarded
        .map(|port| format!("🔌 Port: {}\n", port))
        .unwrap_or_default();

    let change_info = change_details
        .map(|changes| format!("🔄 Changes Detected:\n{}\n\n", changes))
        .unwrap_or_default();

    format!(
        "{} VPN Status Report\n\n{}📍 IP: {}\n🌐 Location: {}\n🔢 ASN: {} ({})\n{}{}{} Status: {}\n⏰ Time: {}",
        status_emoji,
        change_info,
        info.ip.as_deref().unwrap_or("Unknown"),
        location,
        info.asn.as_deref().unwrap_or("Unknown"),
        proton_badge,
        org_info,
        port_info,
        status_emoji,
        status_text,
        timestamp
    )
}

/// Determine notification title based on status
fn determine_title(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    change_details: Option<&str>,
) -> &'static str {
    if change_details.is_some() {
        "🔄 VPN Server Changed!"
    } else {
        let is_allowed = info
            .asn
            .as_ref()
            .is_some_and(|asn| allowed_asns.contains(asn));
        if is_allowed {
            "VPN Health: OK"
        } else {
            "VPN Health: Warning"
        }
    }
}

/// Determine notification priority
fn determine_priority(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    change_details: Option<&str>,
) -> &'static str {
    if change_details.is_some() {
        "high"
    } else {
        let is_allowed = info
            .asn
            .as_ref()
            .is_some_and(|asn| allowed_asns.contains(asn));
        if is_allowed {
            "default"
        } else {
            "high"
        }
    }
}
