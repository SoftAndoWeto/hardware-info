use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_ERROR_NOT_FOUND,
};

use super::{vendor_name, GpuInfo, HwResult};
use super::nvml::{collect_nvml_gpus, enrich_with_nvml};

pub fn get_gpu() -> HwResult<Vec<GpuInfo>> {
    let mut adapters = collect_dxgi_adapters()?;

    if adapters.is_empty() {
        collect_nvml_gpus()
    } else {
        enrich_with_nvml(&mut adapters);
        Ok(adapters)
    }
}

fn collect_dxgi_adapters() -> HwResult<Vec<GpuInfo>> {
    let factory = unsafe {
        CreateDXGIFactory1::<IDXGIFactory1>()
            .map_err(|error| format!("cannot create DXGI factory: {error}"))?
    };

    let mut gpus = Vec::new();
    let mut index = 0;

    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(error) if error.code() == DXGI_ERROR_NOT_FOUND => break,
            Err(error) => return Err(format!("cannot enumerate DXGI adapter #{index}: {error}")),
        };

        let desc = unsafe {
            adapter
                .GetDesc1()
                .map_err(|error| format!("cannot get DXGI adapter #{index} description: {error}"))?
        };

        let dedicated_video_memory = desc.DedicatedVideoMemory as u64;
        let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;

        gpus.push(GpuInfo {
            name: utf16_null_terminated_to_string(&desc.Description),
            virtual_ram: dedicated_video_memory,
            vendor: vendor_name(desc.VendorId).map(str::to_string),
            vendor_id: Some(desc.VendorId),
            device_id: Some(desc.DeviceId),
            dedicated_video_memory: Some(dedicated_video_memory),
            dedicated_system_memory: Some(desc.DedicatedSystemMemory as u64),
            shared_system_memory: Some(desc.SharedSystemMemory as u64),
            is_software: Some(is_software),
            driver_version: None,
            memory_used: None,
            memory_free: None,
            temperature_celsius: None,
            utilization_gpu_percent: None,
            utilization_memory_percent: None,
            power_usage_milliwatts: None,
        });

        index += 1;
    }

    Ok(gpus)
}

fn utf16_null_terminated_to_string(value: &[u16]) -> String {
    let len = value
        .iter()
        .position(|character| *character == 0)
        .unwrap_or(value.len());

    String::from_utf16_lossy(&value[..len]).trim().to_string()
}
