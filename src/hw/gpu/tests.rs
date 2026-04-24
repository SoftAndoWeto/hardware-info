use super::nvml::normalize_gpu_name;

#[test]
fn vendor_name_identifies_known_vendors() {
    assert_eq!(super::vendor_name(0x10DE), Some("NVIDIA"));
    assert_eq!(super::vendor_name(0x1002), Some("AMD"));
    assert_eq!(super::vendor_name(0x8086), Some("Intel"));
    assert_eq!(super::vendor_name(0x1414), Some("Microsoft"));
    assert_eq!(super::vendor_name(0xFFFF), None);
}

#[test]
fn normalize_gpu_name_strips_non_alphanumeric_and_lowercases() {
    assert_eq!(
        normalize_gpu_name("NVIDIA GeForce RTX 4070 Ti SUPER"),
        "nvidiageforcertx4070tisuper"
    );
    assert_eq!(
        normalize_gpu_name("Intel(R) Arc(TM) A770 Graphics"),
        "intelrarctma770graphics"
    );
}

#[test]
fn normalize_gpu_name_empty_string() {
    assert_eq!(normalize_gpu_name(""), "");
}

#[test]
#[cfg(target_os = "linux")]
fn parse_hex_u32_handles_prefixed_and_plain_hex() {
    use super::linux::parse_hex_u32;

    assert_eq!(parse_hex_u32("0x10de"), Some(0x10de));
    assert_eq!(parse_hex_u32("0x8086"), Some(0x8086));
    assert_eq!(parse_hex_u32("10de"), Some(0x10de));
    assert_eq!(parse_hex_u32("0x10DE"), Some(0x10de));
    assert_eq!(parse_hex_u32("invalid"), None);
    assert_eq!(parse_hex_u32(""), None);
}
