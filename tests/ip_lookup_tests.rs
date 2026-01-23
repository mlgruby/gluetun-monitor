// IP lookup module tests
use gluetun_monitor::ip_lookup::gluetun::parse_organization;

#[test]
fn test_parse_organization_with_asn() {
    // Given: Organization string with ASN prefix
    let org = Some("AS212238 Datacamp Limited".to_string());

    // When: Parsing the organization string
    let (asn, org_name) = parse_organization(&org);

    // Then: Should extract ASN and organization name separately
    assert_eq!(asn, Some("AS212238".to_string()));
    assert_eq!(org_name, Some("Datacamp Limited".to_string()));
}

#[test]
fn test_parse_organization_without_asn() {
    // Given: Organization string without ASN prefix
    let org = Some("Just Company Name".to_string());

    // When: Parsing the organization string
    let (asn, org_name) = parse_organization(&org);

    // Then: Function extracts first word as ASN (uppercased), rest as org name
    assert_eq!(asn, Some("JUST".to_string()));
    assert_eq!(org_name, Some("Company Name".to_string()));
}

#[test]
fn test_parse_organization_none() {
    // Given: No organization string provided
    // When: Parsing None
    let (asn, org_name) = parse_organization(&None);

    // Then: Should return None for both fields
    assert_eq!(asn, None);
    assert_eq!(org_name, None);
}

#[test]
fn test_parse_organization_lowercase_asn() {
    // Given: Organization string with lowercase ASN
    let org = Some("as12345 Test Provider".to_string());

    // When: Parsing the organization string
    let (asn, org_name) = parse_organization(&org);

    // Then: ASN should be uppercased
    assert_eq!(asn, Some("AS12345".to_string()));
    assert_eq!(org_name, Some("Test Provider".to_string()));
}

#[test]
fn test_parse_organization_extra_spaces() {
    // Given: Organization string with extra spaces
    let org = Some("AS99999   Multiple   Spaces".to_string());

    // When: Parsing the organization string
    let (asn, org_name) = parse_organization(&org);

    // Then: Should split on first space only, preserving remaining spaces
    assert_eq!(asn, Some("AS99999".to_string()));
    assert_eq!(org_name, Some("Multiple   Spaces".to_string()));
}
