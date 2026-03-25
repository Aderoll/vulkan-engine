use anyhow::{anyhow, Result};
use log::*;
use thiserror::Error;

use std::collections::HashSet;

use vulkanalia::prelude::v1_3::*;

use crate::swapchain::*;
use crate::AppData;

#[derive(Debug, Error)]
#[error("{0}")]
pub struct SuitabilityError(pub &'static str);

pub unsafe fn pick_physical_device(instance: &Instance, data: &mut AppData) -> Result<()> {
    // Obligatory extensions
    data.device_extensions = vec![vk::KHR_SWAPCHAIN_EXTENSION.name];

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
    data.physical_device = valid_devices[valid_devices.len() - 1];

    let optional_extensions: HashSet<vk::ExtensionName> = HashSet::from([
        vk::KHR_SWAPCHAIN_EXTENSION.name,
        vk::KHR_DEDICATED_ALLOCATION_EXTENSION.name,
        vk::KHR_BIND_MEMORY2_EXTENSION.name,
        vk::KHR_MAINTENANCE4_EXTENSION.name,
        vk::KHR_MAINTENANCE5_EXTENSION.name,
        vk::EXT_MEMORY_BUDGET_EXTENSION.name,
        vk::EXT_BUFFER_DEVICE_ADDRESS_EXTENSION.name,
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
        .map(|e| *e) // If seg error look here
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

    Ok(())
}

unsafe fn check_physical_device_extensions(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    extensions: &Vec<vk::ExtensionName>,
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
