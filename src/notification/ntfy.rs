//! ntfy.sh Notification Service
//!
//! Sends rich notifications to ntfy.sh with VPN status information.
//! Includes formatted messages with emojis, priority levels, and tags.

use crate::models::LookupResult;
use reqwest::Client;
use std::collections::HashSet;
use tokio::time::Duration;
use tracing::{error, info, warn};

const MAX_RETRY_ATTEMPTS: u32 = 3;
const BACKOFF_BASE: u64 = 2;

/// Send notification via ntfy with retry logic
pub async fn send_notification(
    client: &Client,
    ntfy_url: &str,
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    allowed_providers: &HashSet<String>,
    home_ip: Option<&str>,
    change_details: Option<&str>,
) -> Result<(), String> {
    let message = build_message(
        info,
        allowed_asns,
        allowed_providers,
        home_ip,
        change_details,
    );
    let title = determine_title(
        info,
        allowed_asns,
        allowed_providers,
        home_ip,
        change_details,
    );
    let priority = determine_priority(
        info,
        allowed_asns,
        allowed_providers,
        home_ip,
        change_details,
    );
    let tags = if info.is_leak(home_ip) {
        "rotating_light,skull,fire"
    } else {
        "vpn,network"
    };

    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        let result = try_send_notification(client, ntfy_url, title, priority, tags, &message).await;

        match result {
            Ok(()) => {
                info!("Notification sent successfully");
                return Ok(());
            }
            Err(err_msg) => {
                if attempt < MAX_RETRY_ATTEMPTS {
                    let backoff_secs = BACKOFF_BASE.pow(attempt);
                    warn!(
                        "{}, retrying in {}s ({}/{})",
                        err_msg, backoff_secs, attempt, MAX_RETRY_ATTEMPTS
                    );
                    tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
                } else {
                    error!(
                        "All {} retry attempts failed: {}",
                        MAX_RETRY_ATTEMPTS, err_msg
                    );
                    return Err(err_msg);
                }
            }
        }
    }

    Err("All retry attempts exhausted".into())
}

async fn try_send_notification(
    client: &Client,
    ntfy_url: &str,
    title: &str,
    priority: &str,
    tags: &str,
    message: &str,
) -> Result<(), String> {
    let response = client
        .post(ntfy_url)
        .header("Title", title)
        .header("Priority", priority)
        .header("Tags", tags)
        .body(message.to_string())
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "Server returned error status: {}",
            response.status()
        ))
    }
}

fn build_message(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    allowed_providers: &HashSet<String>,
    home_ip: Option<&str>,
    change_details: Option<&str>,
) -> String {
    let is_leak = info.is_leak(home_ip);
    let is_allowed = !is_leak && info.is_asn_allowed(allowed_asns, allowed_providers);

    let (status_emoji, status_text) = if is_leak {
        ("🚨", "CRITICAL LEAK DETECTED (Matches Home IP)")
    } else if is_allowed {
        ("✅", "Protected")
    } else {
        ("⚠️", "Unrecognized Provider")
    };

    let org_name = info.org.as_deref().unwrap_or_default().to_uppercase();
    let provider_badge = if is_leak {
        "🚨 HOME ISP LEAK"
    } else if org_name.contains("PROTON") {
        "🔒 Proton VPN"
    } else if org_name.contains("MULLVAD") {
        "🔒 Mullvad VPN"
    } else if is_allowed {
        "🔒 VPN Protected"
    } else {
        "⚡ Unknown Provider"
    };

    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    let org_info = info
        .org
        .as_ref()
        .map(|org| format!("🏢 Provider: {}\n", org))
        .unwrap_or_default();

    let location = info.format_location();

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
        provider_badge,
        org_info,
        port_info,
        status_emoji,
        status_text,
        timestamp
    )
}

fn determine_title(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    allowed_providers: &HashSet<String>,
    home_ip: Option<&str>,
    change_details: Option<&str>,
) -> &'static str {
    if info.is_leak(home_ip) {
        "🚨 CRITICAL: VPN LEAK DETECTED"
    } else if change_details.is_some() {
        "🔄 VPN Server Changed!"
    } else if info.is_asn_allowed(allowed_asns, allowed_providers) {
        "VPN Health: OK"
    } else {
        "VPN Health: Warning"
    }
}

fn determine_priority(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    allowed_providers: &HashSet<String>,
    home_ip: Option<&str>,
    change_details: Option<&str>,
) -> &'static str {
    if info.is_leak(home_ip) {
        "urgent"
    } else if change_details.is_some() {
        "high"
    } else if info.is_asn_allowed(allowed_asns, allowed_providers) {
        "default"
    } else {
        "high"
    }
}
