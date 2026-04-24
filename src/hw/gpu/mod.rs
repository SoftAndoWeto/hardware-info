use serde::{Deserialize, Serialize};

use super::HwResult;

mod nvml;
#[cfg(windows)]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    #[serde(rename = "virtualRam")]
    pub virtual_ram: u64,
    pub vendor: Option<String>,
    #[serde(rename = "vendorId")]
    pub vendor_id: Option<u32>,
    #[serde(rename = "deviceId")]
    pub device_id: Option<u32>,
    #[serde(rename = "dedicatedVideoMemory")]
    pub dedicated_video_memory: Option<u64>,
    #[serde(rename = "dedicatedSystemMemory")]
    pub dedicated_system_memory: Option<u64>,
    #[serde(rename = "sharedSystemMemory")]
    pub shared_system_memory: Option<u64>,
    #[serde(rename = "isSoftware")]
    pub is_software: Option<bool>,
    #[serde(rename = "driverVersion")]
    pub driver_version: Option<String>,
    #[serde(rename = "memoryUsed")]
    pub memory_used: Option<u64>,
    #[serde(rename = "memoryFree")]
    pub memory_free: Option<u64>,
    #[serde(rename = "temperatureCelsius")]
    pub temperature_celsius: Option<u32>,
    #[serde(rename = "utilizationGpuPercent")]
    pub utilization_gpu_percent: Option<u32>,
    #[serde(rename = "utilizationMemoryPercent")]
    pub utilization_memory_percent: Option<u32>,
    #[serde(rename = "powerUsageMilliwatts")]
    pub power_usage_milliwatts: Option<u32>,
}

#[cfg(windows)]
pub use self::windows::get_gpu;
#[cfg(target_os = "linux")]
pub use self::linux::get_gpu;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn get_gpu() -> HwResult<Vec<GpuInfo>> {
    nvml::collect_nvml_gpus()
}

fn vendor_name(vendor_id: u32) -> Option<&'static str> {
    match vendor_id {
        0x10DE => Some("NVIDIA"),
        0x1002 => Some("AMD"),
        0x8086 => Some("Intel"),
        0x1414 => Some("Microsoft"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
