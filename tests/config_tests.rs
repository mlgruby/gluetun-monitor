// Config module tests
use gluetun_monitor::config::Config;
use std::env;

// Note: These tests modify global environment variables and must run serially
// Run with: cargo test -- --test-threads=1

#[test]
fn test_config_from_env_with_defaults() {
    // Given: No environment variables are set
    env::remove_var("VPN_ALLOWED_ASNS");
    env::remove_var("NTFY_URL");
    env::remove_var("GLUETUN_API_URL");
    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");

    // When: Loading config from environment
    let config = Config::from_env();

    // Then: Config should use default values
    assert!(config.allowed_asns.is_empty());
    assert!(config.ntfy_url.is_none());
    assert!(config.gluetun_url.is_none());
    assert_eq!(config.notification_interval_hours, 2);
    assert_eq!(config.check_interval_minutes, 5);
}

#[test]
fn test_config_from_env_with_values() {
    // Given: Environment variables with valid configuration values
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

    // When: Loading config from environment
    let config = Config::from_env();

    // Then: Config should parse all values correctly
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
    // Given: ASN list with extra whitespace and empty entries
    env::set_var("VPN_ALLOWED_ASNS", "  AS12345  ,  , AS67890 ,  ");

    // When: Loading config from environment
    let config = Config::from_env();

    // Then: Should trim whitespace and ignore empty entries
    assert_eq!(config.allowed_asns.len(), 2);
    assert!(config.allowed_asns.contains("AS12345"));
    assert!(config.allowed_asns.contains("AS67890"));

    env::remove_var("VPN_ALLOWED_ASNS");
}

#[test]
fn test_config_minimum_intervals() {
    // Given: Interval values set to 0 (below minimum)
    env::set_var("NTFY_INTERVAL_HOURS", "0");
    env::set_var("VPN_CHECK_INTERVAL_MINUTES", "0");

    // When: Loading config from environment
    let config = Config::from_env();

    // Then: Should enforce minimum value of 1
    assert_eq!(config.notification_interval_hours, 1);
    assert_eq!(config.check_interval_minutes, 1);

    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");
}

#[test]
fn test_config_invalid_interval_values() {
    // Given: Invalid non-numeric interval values
    env::set_var("NTFY_INTERVAL_HOURS", "invalid");
    env::set_var("VPN_CHECK_INTERVAL_MINUTES", "not-a-number");

    // When: Loading config from environment
    let config = Config::from_env();

    // Then: Should fall back to default values
    assert_eq!(config.notification_interval_hours, 2);
    assert_eq!(config.check_interval_minutes, 5);

    env::remove_var("NTFY_INTERVAL_HOURS");
    env::remove_var("VPN_CHECK_INTERVAL_MINUTES");
}
