use serde::{Deserialize, Serialize};

use super::HwResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct BiosInfo {
    pub uuid: String,
    pub manufacturer: String,
    pub name: String,
}

#[cfg(target_os = "windows")]
pub fn get_bios_info() -> HwResult<BiosInfo> {
    let smbios = read_raw_smbios_table()?;
    parse_bios_info_from_smbios(&smbios)
}

#[cfg(not(target_os = "windows"))]
pub fn get_bios_info() -> HwResult<BiosInfo> {
    Err("BIOS collection is only implemented on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn read_raw_smbios_table() -> HwResult<Vec<u8>> {
    use windows::Win32::System::SystemInformation::{GetSystemFirmwareTable, RSMB};

    let size = unsafe { GetSystemFirmwareTable(RSMB, 0, None) };
    if size == 0 {
        return Err("cannot get SMBIOS firmware table size".to_string());
    }

    let mut buffer = vec![0; size as usize];
    let written = unsafe { GetSystemFirmwareTable(RSMB, 0, Some(&mut buffer)) };
    if written == 0 {
        return Err("cannot read SMBIOS firmware table".to_string());
    }

    buffer.truncate(written as usize);
    Ok(buffer)
}

#[cfg(target_os = "windows")]
fn parse_bios_info_from_smbios(raw_smbios: &[u8]) -> HwResult<BiosInfo> {
    let table = smbios_table_bytes(raw_smbios)?;
    let structures = parse_smbios_structures(table);

    let bios = structures
        .iter()
        .find(|structure| structure.structure_type == 0)
        .ok_or_else(|| "SMBIOS type 0 BIOS Information was not found".to_string())?;

    let vendor = bios
        .string_at(bios.formatted_byte(4).unwrap_or_default())
        .ok_or_else(|| "SMBIOS BIOS vendor is missing".to_string())?;
    let version = bios
        .string_at(bios.formatted_byte(5).unwrap_or_default())
        .unwrap_or_default();
    let release_date = bios
        .string_at(bios.formatted_byte(8).unwrap_or_default())
        .unwrap_or_default();
    let uuid = structures
        .iter()
        .find(|structure| structure.structure_type == 1)
        .and_then(|structure| structure.uuid())
        .unwrap_or_default();

    Ok(BiosInfo {
        uuid,
        manufacturer: vendor,
        name: join_non_empty(&[version, release_date]),
    })
}

#[cfg(target_os = "windows")]
fn smbios_table_bytes(raw_smbios: &[u8]) -> HwResult<&[u8]> {
    if raw_smbios.len() < 8 {
        return Err("SMBIOS firmware table is too small".to_string());
    }

    let table_len =
        u32::from_le_bytes([raw_smbios[4], raw_smbios[5], raw_smbios[6], raw_smbios[7]]) as usize;
    let table_start = 8;
    let table_end = table_start + table_len;

    raw_smbios
        .get(table_start..table_end)
        .or_else(|| raw_smbios.get(table_start..))
        .ok_or_else(|| "SMBIOS table data is missing".to_string())
}

#[cfg(target_os = "windows")]
#[derive(Debug)]
struct SmbiosStructure {
    structure_type: u8,
    formatted: Vec<u8>,
    strings: Vec<String>,
}

#[cfg(target_os = "windows")]
impl SmbiosStructure {
    fn formatted_byte(&self, offset: usize) -> Option<u8> {
        self.formatted.get(offset).copied()
    }

    fn string_at(&self, index: u8) -> Option<String> {
        if index == 0 {
            return None;
        }

        self.strings.get(index as usize - 1).cloned()
    }

    fn uuid(&self) -> Option<String> {
        let bytes = self.formatted.get(8..24)?;
        if bytes.iter().all(|byte| *byte == 0) || bytes.iter().all(|byte| *byte == 0xff) {
            return None;
        }

        Some(format!(
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            u16::from_le_bytes([bytes[4], bytes[5]]),
            u16::from_le_bytes([bytes[6], bytes[7]]),
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15],
        ))
    }
}

#[cfg(target_os = "windows")]
fn parse_smbios_structures(table: &[u8]) -> Vec<SmbiosStructure> {
    let mut structures = Vec::new();
    let mut offset = 0;

    while offset + 4 <= table.len() {
        let structure_type = table[offset];
        let length = table[offset + 1] as usize;

        if structure_type == 127 || length < 4 || offset + length > table.len() {
            break;
        }

        let strings_start = offset + length;
        let Some(strings_end) = find_structure_end(table, strings_start) else {
            break;
        };

        structures.push(SmbiosStructure {
            structure_type,
            formatted: table[offset..offset + length].to_vec(),
            strings: parse_smbios_strings(&table[strings_start..strings_end]),
        });

        offset = strings_end + 2;
    }

    structures
}

#[cfg(target_os = "windows")]
fn find_structure_end(table: &[u8], start: usize) -> Option<usize> {
    if start >= table.len() {
        return None;
    }

    table[start..]
        .windows(2)
        .position(|window| window == [0, 0])
        .map(|position| start + position)
}

#[cfg(target_os = "windows")]
fn parse_smbios_strings(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|value| !value.is_empty())
        .map(|value| String::from_utf8_lossy(value).trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

#[cfg(target_os = "windows")]
fn join_non_empty(values: &[String]) -> String {
    values
        .iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn parses_bios_and_system_uuid_from_smbios() {
        let mut table = Vec::new();
        table.extend_from_slice(&[0x00, 0x09, 0x00, 0x00, 0x01, 0x02, 0x00, 0x00, 0x03]);
        table.extend_from_slice(b"American Megatrends International, LLC.\0");
        table.extend_from_slice(b"ALASKA - 1072009\0");
        table.extend_from_slice(b"08/08/2024\0\0");
        table.extend_from_slice(&[
            0x01, 0x19, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67, 0x45, 0x23, 0x01, 0xab, 0x89,
            0xef, 0xcd, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x00,
        ]);
        table.extend_from_slice(b"\0\0");

        let mut raw_smbios = vec![0, 3, 4, 0];
        raw_smbios.extend_from_slice(&(table.len() as u32).to_le_bytes());
        raw_smbios.extend_from_slice(&table);

        let bios = parse_bios_info_from_smbios(&raw_smbios).unwrap();

        assert_eq!(bios.manufacturer, "American Megatrends International, LLC.");
        assert_eq!(bios.name, "ALASKA - 1072009 08/08/2024");
        assert_eq!(bios.uuid, "01234567-89ab-cdef-0123-456789abcdef");
    }
}
