use crate::hw::smbios::read_raw_smbios_table;
use super::{parser::parse_bios_info_from_smbios, BiosInfo, HwResult};

pub fn get_bios_info() -> HwResult<BiosInfo> {
    let smbios = read_raw_smbios_table()?;
    parse_bios_info_from_smbios(&smbios)
}
