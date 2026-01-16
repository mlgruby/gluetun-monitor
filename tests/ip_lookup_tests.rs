// IP lookup module tests
use gluetun_monitor::ip_lookup::gluetun::parse_organization;

#[test]
fn test_parse_organization_with_asn() {
    let org = Some("AS212238 Datacamp Limited".to_string());
    let (asn, org_name) = parse_organization(&org);

    assert_eq!(asn, Some("AS212238".to_string()));
    assert_eq!(org_name, Some("Datacamp Limited".to_string()));
}

#[test]
fn test_parse_organization_without_asn() {
    let org = Some("Just Company Name".to_string());
    let (asn, org_name) = parse_organization(&org);

    // Function extracts first word before space, uppercases it
    assert_eq!(asn, Some("JUST".to_string()));
    assert_eq!(org_name, Some("Company Name".to_string()));
}

#[test]
fn test_parse_organization_none() {
    let (asn, org_name) = parse_organization(&None);

    assert_eq!(asn, None);
    assert_eq!(org_name, None);
}

#[test]
fn test_parse_organization_lowercase_asn() {
    let org = Some("as12345 Test Provider".to_string());
    let (asn, org_name) = parse_organization(&org);

    // Should uppercase the ASN
    assert_eq!(asn, Some("AS12345".to_string()));
    assert_eq!(org_name, Some("Test Provider".to_string()));
}

#[test]
fn test_parse_organization_extra_spaces() {
    let org = Some("AS99999   Multiple   Spaces".to_string());
    let (asn, org_name) = parse_organization(&org);

    assert_eq!(asn, Some("AS99999".to_string()));
    assert_eq!(org_name, Some("Multiple   Spaces".to_string()));
}
