use anyhow::Result;

use vulkanalia::prelude::v1_3::*;

use crate::images::get_supported_format;
use crate::images::{create_image, create_image_view};
use crate::AppData;

pub unsafe fn get_depth_format(instance: &Instance, data: &AppData) -> Result<vk::Format> {
    let candidates = &[
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        instance,
        data,
        candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

pub unsafe fn create_depth_objects(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let format = get_depth_format(instance, data)?;

    let (depth_image, depth_image_memory) = create_image(
        instance,
        device,
        data,
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        1,
        format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.depth_image = depth_image;
    data.depth_image_memory = depth_image_memory;

    // Image View

    data.depth_image_view = create_image_view(
        device,
        data.depth_image,
        format,
        vk::ImageAspectFlags::DEPTH,
        1,
    )?;

    Ok(())
}
