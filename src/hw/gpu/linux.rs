use super::{vendor_name, GpuInfo, HwResult};
use super::nvml::{collect_nvml_gpus, enrich_with_nvml};

pub fn get_gpu() -> HwResult<Vec<GpuInfo>> {
    let mut adapters = match collect_drm_adapters() {
        Ok(adapters) if !adapters.is_empty() => adapters,
        _ => return collect_nvml_gpus(),
    };
    enrich_with_nvml(&mut adapters);
    Ok(adapters)
}

fn collect_drm_adapters() -> HwResult<Vec<GpuInfo>> {
    let mut gpus: Vec<GpuInfo> = std::fs::read_dir("/sys/class/drm")
        .map_err(|e| format!("cannot read /sys/class/drm: {e}"))?
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("card") && name[4..].chars().all(|c| c.is_ascii_digit())
        })
        .filter_map(|entry| parse_drm_card(&entry.path()))
        .collect();

    gpus.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(gpus)
}

fn parse_drm_card(path: &std::path::Path) -> Option<GpuInfo> {
    let vendor_id = read_hex_u32(&path.join("device/vendor"))?;
    let device_id = read_hex_u32(&path.join("device/device")).unwrap_or(0);

    let name = read_sysfs_string(&path.join("device/label"))
        .unwrap_or_else(|| format_gpu_name(vendor_id, device_id));

    let dedicated_video_memory = read_sysfs_u64(&path.join("device/mem_info_vram_total"));

    Some(GpuInfo {
        name,
        virtual_ram: dedicated_video_memory.unwrap_or(0),
        vendor: vendor_name(vendor_id).map(str::to_string),
        vendor_id: Some(vendor_id),
        device_id: Some(device_id),
        dedicated_video_memory,
        dedicated_system_memory: None,
        shared_system_memory: None,
        is_software: None,
        driver_version: None,
        memory_used: None,
        memory_free: None,
        temperature_celsius: None,
        utilization_gpu_percent: None,
        utilization_memory_percent: None,
        power_usage_milliwatts: None,
    })
}

fn format_gpu_name(vendor_id: u32, device_id: u32) -> String {
    match vendor_name(vendor_id) {
        Some(name) => format!("{name} [{device_id:#06x}]"),
        None => format!("Unknown GPU [{vendor_id:#06x}:{device_id:#06x}]"),
    }
}

pub(super) fn parse_hex_u32(s: &str) -> Option<u32> {
    let hex = s.trim().trim_start_matches("0x");
    u32::from_str_radix(hex, 16).ok()
}

fn read_hex_u32(path: &std::path::Path) -> Option<u32> {
    parse_hex_u32(&read_sysfs_string(path)?)
}

fn read_sysfs_string(path: &std::path::Path) -> Option<String> {
    let value = std::fs::read_to_string(path).ok()?;
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() { None } else { Some(trimmed) }
}

fn read_sysfs_u64(path: &std::path::Path) -> Option<u64> {
    read_sysfs_string(path)?.parse().ok()
}
