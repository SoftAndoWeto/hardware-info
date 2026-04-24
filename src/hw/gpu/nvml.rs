use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Nvml};

use super::{GpuInfo, HwResult};

pub(super) struct NvmlGpuInfo {
    pub name: String,
    pub memory_total: Option<u64>,
    pub memory_used: Option<u64>,
    pub memory_free: Option<u64>,
    pub temperature_celsius: Option<u32>,
    pub utilization_gpu_percent: Option<u32>,
    pub utilization_memory_percent: Option<u32>,
    pub power_usage_milliwatts: Option<u32>,
}

pub(super) fn collect_nvml_gpus() -> HwResult<Vec<GpuInfo>> {
    let (driver_version, devices) = collect_nvml_device_info()?;

    Ok(devices
        .into_iter()
        .map(|device| {
            let memory_total = device.memory_total.unwrap_or_default();

            GpuInfo {
                name: device.name,
                virtual_ram: memory_total,
                vendor: Some("NVIDIA".to_string()),
                vendor_id: Some(0x10DE),
                device_id: None,
                dedicated_video_memory: Some(memory_total),
                dedicated_system_memory: None,
                shared_system_memory: None,
                is_software: None,
                driver_version: driver_version.clone(),
                memory_used: device.memory_used,
                memory_free: device.memory_free,
                temperature_celsius: device.temperature_celsius,
                utilization_gpu_percent: device.utilization_gpu_percent,
                utilization_memory_percent: device.utilization_memory_percent,
                power_usage_milliwatts: device.power_usage_milliwatts,
            }
        })
        .collect())
}

pub(super) fn collect_nvml_device_info() -> HwResult<(Option<String>, Vec<NvmlGpuInfo>)> {
    let nvml = Nvml::init().map_err(|error| format!("cannot initialize NVML: {error}"))?;

    let device_count = nvml
        .device_count()
        .map_err(|error| format!("cannot get GPU count: {error}"))?;

    let driver_version = nvml.sys_driver_version().ok();
    let mut devices = Vec::with_capacity(device_count as usize);

    for i in 0..device_count {
        let device = nvml
            .device_by_index(i)
            .map_err(|error| format!("cannot access GPU #{i}: {error}"))?;

        let memory = device.memory_info().ok();
        let utilization = device.utilization_rates().ok();

        devices.push(NvmlGpuInfo {
            name: device
                .name()
                .map_err(|error| format!("cannot get GPU #{i} name: {error}"))?,
            memory_total: memory.as_ref().map(|m| m.total),
            memory_used: memory.as_ref().map(|m| m.used),
            memory_free: memory.as_ref().map(|m| m.free),
            temperature_celsius: device.temperature(TemperatureSensor::Gpu).ok(),
            utilization_gpu_percent: utilization.as_ref().map(|u| u.gpu),
            utilization_memory_percent: utilization.as_ref().map(|u| u.memory),
            power_usage_milliwatts: device.power_usage().ok(),
        });
    }

    Ok((driver_version, devices))
}

pub(super) fn enrich_with_nvml(gpus: &mut [GpuInfo]) {
    let Ok((driver_version, devices)) = collect_nvml_device_info() else {
        return;
    };

    for device in devices {
        let Some(gpu) = find_nvml_target(gpus, &device.name) else {
            continue;
        };

        gpu.name = device.name;

        if let Some(memory_total) = device.memory_total {
            gpu.virtual_ram = memory_total;
            gpu.dedicated_video_memory = Some(memory_total);
        }

        gpu.driver_version = driver_version.clone();
        gpu.memory_used = device.memory_used;
        gpu.memory_free = device.memory_free;
        gpu.temperature_celsius = device.temperature_celsius;
        gpu.utilization_gpu_percent = device.utilization_gpu_percent;
        gpu.utilization_memory_percent = device.utilization_memory_percent;
        gpu.power_usage_milliwatts = device.power_usage_milliwatts;
    }
}

fn find_nvml_target<'a>(gpus: &'a mut [GpuInfo], nvml_name: &str) -> Option<&'a mut GpuInfo> {
    let normalized_nvml_name = normalize_gpu_name(nvml_name);
    let name_match = gpus.iter().position(|gpu| {
        gpu.vendor_id == Some(0x10DE)
            && gpu.driver_version.is_none()
            && normalize_gpu_name(&gpu.name) == normalized_nvml_name
    });

    let fallback_match = || {
        gpus.iter()
            .position(|gpu| gpu.vendor_id == Some(0x10DE) && gpu.driver_version.is_none())
    };

    let index = name_match.or_else(fallback_match)?;
    gpus.get_mut(index)
}

pub(super) fn normalize_gpu_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
