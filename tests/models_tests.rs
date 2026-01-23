// Models module tests
use gluetun_monitor::models::{CheckResponse, LookupResult, StatusResponse};
use std::sync::Arc;

#[test]
fn test_lookup_result_serialization() {
    // Given: A complete LookupResult with all fields populated
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

    // When: Serializing to JSON
    let json = serde_json::to_string(&result).unwrap();

    // Then: Should contain all populated fields and omit None fields
    assert!(json.contains("\"ip\":\"1.2.3.4\""));
    assert!(json.contains("\"asn\":\"AS12345\""));
    assert!(!json.contains("\"error\"")); // Should be omitted when None
}

#[test]
fn test_lookup_result_with_error() {
    // Given: A LookupResult with only error field populated
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

    // When: Serializing to JSON
    let json = serde_json::to_string(&result).unwrap();

    // Then: Should contain error and omit all None fields
    assert!(json.contains("\"error\":\"Lookup failed\""));
    assert!(!json.contains("\"ip\"")); // Should be omitted when None
}

#[test]
fn test_status_response_serialization() {
    // Given: A StatusResponse with lookup data and configuration
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

    // When: Serializing to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Then: Should flatten lookup fields and include config fields
    assert!(json.contains("\"ip\":\"1.2.3.4\""));
    assert!(json.contains("\"allowed_asns\""));
    assert!(json.contains("\"configured\":true"));
}

#[test]
fn test_check_response_ok() {
    // Given: A successful check response with allowed ASN
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

    // When: Serializing to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Then: Should indicate success and omit reason field
    assert!(json.contains("\"ok\":true"));
    assert!(!json.contains("\"reason\"")); // Should be omitted when None
}

#[test]
fn test_check_response_not_ok() {
    // Given: A failed check response with disallowed ASN
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

    // When: Serializing to JSON
    let json = serde_json::to_string(&response).unwrap();

    // Then: Should indicate failure and include reason
    assert!(json.contains("\"ok\":false"));
    assert!(json.contains("\"reason\":\"ASN not allowed\""));
}

use std::collections::HashSet;

#[test]
fn test_is_asn_allowed_when_in_set() {
    // Given: A lookup result with an ASN
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

    // When: Checking if ASN is allowed
    let is_allowed = result.is_asn_allowed(&allowed_asns);

    // Then: It should be allowed
    assert!(is_allowed);
}

#[test]
fn test_is_asn_allowed_when_not_in_set() {
    // Given: A lookup result with an ASN not in the allowed set
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: Some("AS99999".to_string()),
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let mut allowed_asns = HashSet::new();
    allowed_asns.insert("AS12345".to_string());

    // When: Checking if ASN is allowed
    let is_allowed = result.is_asn_allowed(&allowed_asns);

    // Then: It should NOT be allowed
    assert!(!is_allowed);
}

#[test]
fn test_is_asn_allowed_when_asn_is_none() {
    // Given: A lookup result without an ASN
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: None,
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    let mut allowed_asns = HashSet::new();
    allowed_asns.insert("AS12345".to_string());

    // When: Checking if ASN is allowed
    let is_allowed = result.is_asn_allowed(&allowed_asns);

    // Then: It should NOT be allowed (no ASN means not allowed)
    assert!(!is_allowed);
}

#[test]
fn test_is_asn_allowed_when_empty_set() {
    // Given: A lookup result with an ASN but empty allowed set
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

    // When: Checking if ASN is allowed
    let is_allowed = result.is_asn_allowed(&allowed_asns);

    // Then: It should NOT be allowed (empty set means nothing is allowed)
    assert!(!is_allowed);
}

#[test]
fn test_format_location_city_and_country() {
    // Given: A result with both city and country
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

    // When: Formatting location
    let location = result.format_location();

    // Then: Should return "City, Country"
    assert_eq!(location, "New York, United States");
}

#[test]
fn test_format_location_country_only() {
    // Given: A result with only country
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: None,
        org: None,
        country: Some("United States".to_string()),
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    // When: Formatting location
    let location = result.format_location();

    // Then: Should return just the country
    assert_eq!(location, "United States");
}

#[test]
fn test_format_location_city_only() {
    // Given: A result with only city
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: None,
        org: None,
        country: None,
        city: Some("New York".to_string()),
        region: None,
        port_forwarded: None,
        error: None,
    };

    // When: Formatting location
    let location = result.format_location();

    // Then: Should return just the city
    assert_eq!(location, "New York");
}

#[test]
fn test_format_location_region_only() {
    // Given: A result with only region
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: None,
        org: None,
        country: None,
        city: None,
        region: Some("California".to_string()),
        port_forwarded: None,
        error: None,
    };

    // When: Formatting location
    let location = result.format_location();

    // Then: Should return the region
    assert_eq!(location, "California");
}

#[test]
fn test_format_location_all_none_returns_unknown() {
    // Given: A result with no location data
    let result = LookupResult {
        ip: Some("1.2.3.4".to_string()),
        asn: None,
        org: None,
        country: None,
        city: None,
        region: None,
        port_forwarded: None,
        error: None,
    };

    // When: Formatting location
    let location = result.format_location();

    // Then: Should return "Unknown"
    assert_eq!(location, "Unknown");
}
