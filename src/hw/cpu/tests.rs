use super::*;

#[test]
fn normalizes_cpu_brand_by_trimming_whitespace() {
    let raw = "  Intel(R) Core(TM) i7-14700K  ";
    let normalized = normalize_cpu_brand(raw);
    assert_eq!(normalized, "Intel(R) Core(TM) i7-14700K");
}

#[test]
fn composes_cpu_identifier_from_vendor_and_name() {
    let identifier = compose_cpu_identifier("GenuineIntel", "14th Gen Intel(R) Core(TM)");
    assert_eq!(identifier, "GenuineIntel - 14th Gen Intel(R) Core(TM)");
}

#[test]
fn converts_mhz_to_hz() {
    assert_eq!(mhz_to_hz(3200), 3_200_000_000);
}

#[test]
fn converts_mhz_to_hz_saturates_on_overflow() {
    assert_eq!(mhz_to_hz(u64::MAX), u64::MAX);
}
