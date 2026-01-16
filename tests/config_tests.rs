// Config module tests
use gluetun_monitor::config::Config;
use std::env;

// Note: These tests modify global environment variables and must run serially
// Run with: cargo test -- --test-threads=1

#[test]
fn test_config_from_env_with_defaults() {
    // Clear environment
    env::remove_var("VPN_ALLOWED_ASNS");
    env::remove_var("NTFY_URL");
    env::remove_var("GLUETUN_API_URL");
    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");

    let config = Config::from_env();

    assert!(config.allowed_asns.is_empty());
    assert!(config.ntfy_url.is_none());
    assert!(config.gluetun_url.is_none());
    assert_eq!(config.notification_interval_hours, 2);
    assert_eq!(config.check_interval_minutes, 5);
}

#[test]
fn test_config_from_env_with_values() {
    // Clear potentially conflicting env vars first
    env::remove_var("VPN_ALLOWED_ASNS");
    env::remove_var("NTFY_URL");
    env::remove_var("GLUETUN_API_URL");
    env::remove_var("GLUETUN_API_KEY");
    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");

    env::set_var("VPN_ALLOWED_ASNS", "AS12345,AS67890,as99999");
    env::set_var("NTFY_URL", "https://ntfy.sh/test");
    env::set_var("GLUETUN_API_URL", "http://localhost:8000");
    env::set_var("GLUETUN_API_KEY", "test-key");
    env::set_var("NTFY_INTERVAL_HOURS", "3");
    env::set_var("VPN_CHECK_INTERVAL_MINUTES", "10");

    let config = Config::from_env();

    assert_eq!(config.allowed_asns.len(), 3);
    assert!(config.allowed_asns.contains("AS12345"));
    assert!(config.allowed_asns.contains("AS67890"));
    assert!(config.allowed_asns.contains("AS99999")); // Should be uppercased
    assert_eq!(config.ntfy_url, Some("https://ntfy.sh/test".to_string()));
    assert_eq!(
        config.gluetun_url,
        Some("http://localhost:8000".to_string())
    );
    assert_eq!(config.gluetun_api_key, Some("test-key".to_string()));
    assert_eq!(config.notification_interval_hours, 3);
    assert_eq!(config.check_interval_minutes, 10);

    // Cleanup
    env::remove_var("VPN_ALLOWED_ASNS");
    env::remove_var("NTFY_URL");
    env::remove_var("GLUETUN_API_URL");
    env::remove_var("GLUETUN_API_KEY");
    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");
}

#[test]
fn test_config_asn_parsing() {
    env::set_var("VPN_ALLOWED_ASNS", "  AS12345  ,  , AS67890 ,  ");

    let config = Config::from_env();

    assert_eq!(config.allowed_asns.len(), 2);
    assert!(config.allowed_asns.contains("AS12345"));
    assert!(config.allowed_asns.contains("AS67890"));

    env::remove_var("VPN_ALLOWED_ASNS");
}

#[test]
fn test_config_minimum_intervals() {
    env::set_var("NTFY_INTERVAL_HOURS", "0");
    env::set_var("VPN_CHECK_INTERVAL_MINUTES", "0");

    let config = Config::from_env();

    // Should enforce minimum of 1
    assert_eq!(config.notification_interval_hours, 1);
    assert_eq!(config.check_interval_minutes, 1);

    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");
}

#[test]
fn test_config_invalid_interval_values() {
    env::set_var("NTFY_INTERVAL_HOURS", "invalid");
    env::set_var("VPN_CHECK_INTERVAL_MINUTES", "not-a-number");

    let config = Config::from_env();

    // Should fall back to defaults
    assert_eq!(config.notification_interval_hours, 2);
    assert_eq!(config.check_interval_minutes, 5);

    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");
}
