// Models module tests
use gluetun_monitor::models::{CheckResponse, LookupResult, StatusResponse};

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
    assert!(!json.contains("\"error\"")); // Should be omitted when None
}

#[test]
fn test_lookup_result_with_error() {
    let result = LookupResult {
        ip: None,
        asn: None,
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: Some("Lookup failed".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"error\":\"Lookup failed\""));
    assert!(!json.contains("\"ip\"")); // Should be omitted when None
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

    let response = StatusResponse {
        lookup,
        allowed_asns: vec!["AS12345".to_string(), "AS67890".to_string()],
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
    assert!(!json.contains("\"reason\"")); // Should be omitted when None
}

#[test]
fn test_check_response_not_ok() {
    let lookup = LookupResult {
        ip: Some("5.6.7.8".to_string()),
        asn: Some("AS99999".to_string()),
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let response = CheckResponse {
        ok: false,
        reason: Some("ASN not allowed".to_string()),
        lookup,
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"ok\":false"));
    assert!(json.contains("\"reason\":\"ASN not allowed\""));
}
