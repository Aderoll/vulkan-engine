use anyhow::{anyhow, Result};
use log::*;
use thiserror::Error;

use std::collections::HashSet;

use vulkanalia::prelude::v1_3::*;
// use vulkanalia::vk::EXT_DESCRIPTOR_BUFFER_EXTENSION;

use crate::swapchain::*;
use crate::AppData;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct SuitabilityError(pub &'static str);

pub unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    // Obligatory extensions
    data.device_extensions = vec![
        vk::KHR_SWAPCHAIN_EXTENSION.name,
        // EXT_DESCRIPTOR_BUFFER_EXTENSION.name, TODO
    ];

    let mut valid_devices = vec![];
    for physical_device in instance.enumerate_physical_devices()? {
        let properties = instance.get_physical_device_properties(physical_device);

        if let Err(error) = check_physical_device(instance, data, physical_device) {
            warn!(
                "Skipping physical device (`{}`): {}",
                properties.device_name, error
            );
        } else {
            info!("Valid physical device (`{}`).", properties.device_name);
            valid_devices.push(physical_device);
        }
    }
    //data.physical_device = valid_devices[valid_devices.len() - 1];
    data.physical_device = valid_devices[0];
    data.msaa_samples = get_max_msaa_samples(instance, data);

    let properties = instance.get_physical_device_properties(data.physical_device);

    info!("Using device (`{}`)", properties.device_name);
    info!("Using MSAA samples: {:?}", data.msaa_samples);

    let optional_extensions: HashSet<vk::ExtensionName> = HashSet::from([
        vk::KHR_SWAPCHAIN_EXTENSION.name,
        vk::KHR_DEDICATED_ALLOCATION_EXTENSION.name,
        vk::KHR_BIND_MEMORY2_EXTENSION.name,
        vk::KHR_MAINTENANCE4_EXTENSION.name,
        vk::KHR_MAINTENANCE5_EXTENSION.name,
        vk::EXT_MEMORY_BUDGET_EXTENSION.name,
        vk::KHR_BUFFER_DEVICE_ADDRESS_EXTENSION.name,
        vk::EXT_MEMORY_PRIORITY_EXTENSION.name,
        vk::AMD_DEVICE_COHERENT_MEMORY_EXTENSION.name,
        vk::KHR_EXTERNAL_MEMORY_WIN32_EXTENSION.name,
    ]);

    let compatible_extensions = instance
        .enumerate_device_extension_properties(data.physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();

    data.device_extensions = compatible_extensions
        .intersection(&optional_extensions)
        .copied()
        .collect();

    Ok(())
}

unsafe fn check_physical_device(
    instance: &Instance,
    data: &AppData,
    physical_device: vk::PhysicalDevice,
) -> Result<()> {
    QueueFamilyIndices::get(instance, data, physical_device)?;
    check_physical_device_extensions(instance, physical_device, &data.device_extensions)?;

    let support = SwapchainSupport::get(instance, data, physical_device)?;
    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
    }

    let features = instance.get_physical_device_features(physical_device);
    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!(SuitabilityError("No sampler anisotropy.")));
    }
    let properties = instance.get_physical_device_properties(physical_device);
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(anyhow!(SuitabilityError("Is not a DISCRETE_GPU")));
    }

    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    extensions: &[vk::ExtensionName],
) -> Result<()> {
    let extensions = instance
        .enumerate_device_extension_properties(physical_device, None)?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
    if extensions.iter().all(|e| extensions.contains(e)) {
        Ok(())
    } else {
        Err(anyhow!(SuitabilityError(
            "Missing required device extensions."
        )))
    }
}

pub unsafe fn get_max_msaa_samples(instance: &Instance, data: &AppData) -> vk::SampleCountFlags {
    let properties = instance.get_physical_device_properties(data.physical_device);
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}

pub unsafe fn create_logical_device(
    entry: &Entry,
    instance: &Instance,
    data: &mut AppData,
) -> Result<Device> {
    // Queue Create Infos

    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    let queue_priorities = &[1.0];
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(*i)
                .queue_priorities(queue_priorities)
        })
        .collect::<Vec<_>>();

    // Layers

    let layers = if data.validation_enabled {
        vec![data.validation_layer.as_ptr()]
    } else {
        vec![]
    };

    // Extensions

    let extensions = data
        .device_extensions
        .iter()
        .map(|n| n.as_ptr())
        .collect::<Vec<_>>();

    // Features

    let features = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);

    // Create

    let info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .enabled_features(&features);

    let device = instance.create_device(data.physical_device, &info, None)?;

    // Queues

    data.graphics_queue = device.get_device_queue(indices.graphics, 0);
    data.present_queue = device.get_device_queue(indices.present, 0);

    Ok(device)
}
