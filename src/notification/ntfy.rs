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

/// Maximum number of retry attempts for failed notifications
const MAX_RETRY_ATTEMPTS: u32 = 3;

/// Base for exponential backoff calculation (backoff = BASE^attempt seconds)
/// Example: attempt 1 = 2^1 = 2s, attempt 2 = 2^2 = 4s, attempt 3 = 2^3 = 8s
const BACKOFF_BASE: u64 = 2;

/// Send notification via ntfy with retry logic
///
/// **Retry Strategy:**
/// - Attempts: 3 (defined by MAX_RETRY_ATTEMPTS)
/// - Backoff: Exponential (2^attempt seconds: 2s, 4s, 8s)
/// - Returns: Ok on success, Err with message on failure
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

    // Try sending with exponential backoff retry
    for attempt in 1..=MAX_RETRY_ATTEMPTS {
        let result = try_send_notification(client, ntfy_url, title, priority, &message).await;

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

/// Attempt to send a single notification (helper for retry logic)
async fn try_send_notification(
    client: &Client,
    ntfy_url: &str,
    title: &str,
    priority: &str,
    message: &str,
) -> Result<(), String> {
    let response = client
        .post(ntfy_url)
        .header("Title", title)
        .header("Priority", priority)
        .header("Tags", "vpn,network")
        .body(message.to_string()) // Allocate owned String for request body lifetime
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

/// Build notification message with status and details
fn build_message(
    info: &LookupResult,
    allowed_asns: &HashSet<String>,
    change_details: Option<&str>,
) -> String {
    let is_allowed = info.is_asn_allowed(allowed_asns);
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

    // Use extracted location formatter
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
    } else if info.is_asn_allowed(allowed_asns) {
        "VPN Health: OK"
    } else {
        "VPN Health: Warning"
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
    } else if info.is_asn_allowed(allowed_asns) {
        "default"
    } else {
        "high"
    }
}
