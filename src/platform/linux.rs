use crate::{HardwareInfo, HardwareInfoError, OsInfo, Result};

pub(crate) fn collect() -> Result<HardwareInfo> {
    Err(HardwareInfoError::PlatformUnsupported("linux"))
}

pub(crate) fn collect_os_info() -> Result<OsInfo> {
    Ok(collect_os())
}

fn collect_os() -> OsInfo {
    let (name, version) = parse_os_release();

    OsInfo {
        family: std::env::consts::FAMILY.to_string(),
        name,
        version,
    }
}

fn parse_os_release() -> (Option<String>, Option<String>) {
    let content = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let mut name = None;
    let mut version = None;

    for line in content.lines() {
        if let Some(value) = line.strip_prefix("NAME=") {
            name = Some(strip_quotes(value).to_string());
        } else if let Some(value) = line.strip_prefix("VERSION_ID=") {
            version = Some(strip_quotes(value).to_string());
        }
        if name.is_some() && version.is_some() {
            break;
        }
    }

    (name, version)
}

fn strip_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_os_release() {
        let content = r#"NAME="Ubuntu"
VERSION_ID="22.04"
ID=ubuntu
"#;
        let (name, version) = parse_os_release_str(content);
        assert_eq!(name, Some("Ubuntu".to_string()));
        assert_eq!(version, Some("22.04".to_string()));
    }

    #[test]
    fn parses_unquoted_os_release() {
        let content = "NAME=Arch Linux\nVERSION_ID=rolling\n";
        let (name, version) = parse_os_release_str(content);
        assert_eq!(name, Some("Arch Linux".to_string()));
        assert_eq!(version, Some("rolling".to_string()));
    }

    #[test]
    fn handles_missing_version_id() {
        let content = "NAME=\"Arch Linux\"\nID=arch\n";
        let (name, version) = parse_os_release_str(content);
        assert_eq!(name, Some("Arch Linux".to_string()));
        assert_eq!(version, None);
    }

    #[test]
    fn handles_empty_os_release() {
        let (name, version) = parse_os_release_str("");
        assert_eq!(name, None);
        assert_eq!(version, None);
    }

    fn parse_os_release_str(content: &str) -> (Option<String>, Option<String>) {
        let mut name = None;
        let mut version = None;

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("NAME=") {
                name = Some(strip_quotes(value).to_string());
            } else if let Some(value) = line.strip_prefix("VERSION_ID=") {
                version = Some(strip_quotes(value).to_string());
            }
            if name.is_some() && version.is_some() {
                break;
            }
        }

        (name, version)
    }
}
