use gluetun_monitor::models::{CheckResponse, LookupResult, StatusResponse};
use std::collections::HashSet;
use std::sync::Arc;

#[test]
fn test_lookup_result_serialization() {
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: Some("AS12345".to_string()),
        org: Some("Test Org".to_string()),
        country: Some("Netherlands".to_string()),
        city: Some("Amsterdam".to_string()),
        region: Some("North Holland".to_string()),
        port_forwarded: Some(54321),
        error: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"ip\":\"1.2.3.4\""));
    assert!(json.contains("\"asn\":\"AS12345\""));
    assert!(!json.contains("\"error\""));
}

#[test]
fn test_status_response_serialization() {
    let lookup = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: Some("AS12345".to_string()),
        org: Some("Test Org".to_string()),
        country: Some("Netherlands".to_string()),
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let allowed_asns = Arc::new(vec!["AS12345".to_string(), "AS67890".to_string()]);
    let response = StatusResponse {
        lookup,
        allowed_asns,
        configured: true,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"ip\":\"1.2.3.4\""));
    assert!(json.contains("\"allowed_asns\""));
    assert!(json.contains("\"configured\":true"));
}

#[test]
fn test_check_response_ok() {
    let lookup = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: Some("AS12345".to_string()),
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let response = CheckResponse {
        ok: true,
        reason: None,
        lookup,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"ok\":true"));
    assert!(!json.contains("\"reason\""));
}

#[test]
fn test_is_asn_allowed_when_in_set() {
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: Some("AS12345".to_string()),
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let mut allowed_asns = HashSet::new();
    allowed_asns.insert("AS12345".to_string());
    let allowed_providers = HashSet::new();

    assert!(result.is_asn_allowed(&allowed_asns, &allowed_providers));
}

#[test]
fn test_is_asn_allowed_when_provider_matches() {
    let result = LookupResult {
        ip: Some("185.159.157.75".to_string()),
        asn: Some("AS209103".to_string()),
        org: Some("Proton AG".to_string()),
        country: Some("Switzerland".to_string()),
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let allowed_asns = HashSet::new();
    let mut allowed_providers = HashSet::new();
    allowed_providers.insert("PROTON".to_string());

    assert!(result.is_asn_allowed(&allowed_asns, &allowed_providers));
}

#[test]
fn test_is_asn_allowed_when_empty_set() {
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: Some("AS12345".to_string()),
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let allowed_asns = HashSet::new();
    let allowed_providers = HashSet::new();

    assert!(result.is_asn_allowed(&allowed_asns, &allowed_providers));
}

#[test]
fn test_is_leak_detection() {
    let leaking_result = LookupResult {
        ip: Some("82.10.20.30".to_string()),
        asn: Some("AS5089".to_string()),
        org: Some("Virgin Media".to_string()),
        country: Some("UK".to_string()),
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    assert!(leaking_result.is_leak(Some("82.10.20.30")));
    assert!(!leaking_result.is_leak(Some("185.159.157.75")));
    assert!(!leaking_result.is_leak(None));
}

#[test]
fn test_format_location() {
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: None,
        org: None,
        country: Some("United States".to_string()),
        city: Some("New York".to_string()),
        region: None,
        port_forwarded: None,
        error: None,
    };

    assert_eq!(result.format_location(), "New York, United States");
}
