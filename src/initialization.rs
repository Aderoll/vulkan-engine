use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::time::Instant;

use crate::command::{create_command_buffers, create_command_pool};
use crate::create_uniform_buffers;
use crate::debug::debug_callback;
use crate::descriptors::{
    create_descriptor_pool, create_descriptor_set_layout, create_descriptor_sets,
};
use crate::device::pick_physical_device;
use crate::framebuffer::create_framebuffers;
use crate::images::create_texture_image;
use crate::memory::{create_index_buffer, create_vertex_buffer};
use crate::pipeline::{create_pipeline, create_render_pass};
use crate::swapchain::{create_logical_device, create_swapchain, create_swapchain_image_views};
use crate::sync::create_sync_objects;
use crate::{App, AppData, Instance};

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_3::*;
use vulkanalia::vk::ExtDebugUtilsExtensionInstanceCommands;
use vulkanalia::window as vk_window;
use winit::window::Window;

unsafe fn create_instance(window: &Window, entry: &Entry, data: &mut AppData) -> Result<Instance> {
    // Application Info

    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial (Rust)\0")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 3, 216));

    // Layers

    let validation_enabled = cfg!(debug_assertions);

    let validation_layer = vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if validation_enabled && !available_layers.contains(&validation_layer) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if validation_enabled {
        vec![validation_layer.as_ptr()]
    } else {
        Vec::new()
    };

    // Extensions

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    let flags = vk::InstanceCreateFlags::empty();

    if validation_enabled {
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // Create

    let mut info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .user_callback(Some(debug_callback));

    if validation_enabled {
        info = info.push_next(&mut debug_info);
    }

    let instance = entry.create_instance(&info, None)?;

    data.validation_enabled = validation_enabled;
    data.validation_layer = validation_layer;

    let device_extensions: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

    // Messenger

    if validation_enabled {
        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}

/// Creates the Vulkan app.
pub unsafe fn create_app(window: &Window) -> Result<App> {
    let loader = LibloadingLoader::new(LIBRARY)?;
    let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
    let mut data = AppData {
        max_frames_in_flight: 2, // TODO: Make better way to change it
        ..Default::default()
    };
    let instance = create_instance(window, &entry, &mut data)?;
    data.surface = vk_window::create_surface(&instance, &window, &window)?;
    pick_physical_device(&instance, &mut data)?;
    let device = create_logical_device(&entry, &instance, &mut data)?;
    create_swapchain(window, &instance, &device, &mut data)?;
    create_swapchain_image_views(&device, &mut data)?;
    create_render_pass(&instance, &device, &mut data)?;
    create_descriptor_set_layout(&device, &mut data)?;
    create_pipeline(&device, &mut data)?;
    create_framebuffers(&device, &mut data)?;
    create_command_pool(&instance, &device, &mut data)?;
    create_texture_image(&instance, &device, &mut data)?;
    create_vertex_buffer(&instance, &device, &mut data)?;
    create_index_buffer(&instance, &device, &mut data)?;
    create_uniform_buffers(&instance, &device, &mut data)?;
    create_descriptor_pool(&device, &mut data)?;
    create_descriptor_sets(&device, &mut data)?;
    create_command_buffers(&device, &mut data)?;
    create_sync_objects(&device, &mut data)?;
    Ok(App {
        entry,
        instance,
        data,
        device,
        frame: 0,
        resized: false,
        start: Instant::now(),
    })
}
