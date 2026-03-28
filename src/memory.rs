use anyhow::{anyhow, Result};

use std::ptr::copy_nonoverlapping as memcpy;

use vulkanalia::prelude::v1_3::*;

use crate::descriptors::Mat4;
use crate::descriptors::UniformBufferObject;
use crate::vertices::{Vertex, INDICES, VERTICES};
use crate::{App, AppData};

use cgmath::{point3, vec3, Deg};

pub unsafe fn get_memory_type_index(
    instance: &Instance,
    data: &AppData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memory = instance.get_physical_device_memory_properties(data.physical_device);
    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

pub unsafe fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = device.create_buffer(&buffer_info, None)?;

    let requirements = device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            data,
            properties,
            requirements,
        )?);

    let buffer_memory = device.allocate_memory(&memory_info, None)?;

    device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

pub unsafe fn create_buffer_staged<T>(
    source: &[T],
    instance: &Instance,
    device: &Device,
    data: &AppData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    memcpy(source.as_ptr(), memory.cast(), source.len());

    device.unmap_memory(staging_buffer_memory);

    let (buffer, buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        usage | vk::BufferUsageFlags::TRANSFER_DST,
        properties,
    )?;

    copy_buffer(device, data, staging_buffer, buffer, size)?;

    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((buffer, buffer_memory))
}

unsafe fn copy_buffer(
    device: &Device,
    data: &AppData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_pool)
        .command_buffer_count(1);

    let command_buffer = device.allocate_command_buffers(&info)?[0];

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    device.begin_command_buffer(command_buffer, &info)?;

    let regions = vk::BufferCopy::builder().size(size);
    device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

    device.end_command_buffer(command_buffer)?;

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    device.queue_submit(data.graphics_queue, &[info], vk::Fence::null())?;
    device.queue_wait_idle(data.graphics_queue)?;

    device.free_command_buffers(data.command_pool, &[command_buffer]);
    Ok(())
}

pub unsafe fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<Vertex>() * VERTICES.len()) as u64;

    let (vertex_buffer, vertex_buffer_memory) = create_buffer_staged(
        VERTICES.as_slice(),
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )
    .expect("Could not create Vertex Buffer");

    data.vertex_buffer = vertex_buffer;
    data.vertex_buffer_memory = vertex_buffer_memory;

    Ok(())
}

pub unsafe fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let size = (size_of::<u16>() * INDICES.len()) as u64;

    let (index_buffer, index_buffer_memory) = create_buffer_staged(
        INDICES,
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )
    .expect("Could not create Index Buffer");

    data.index_buffer = index_buffer;
    data.index_buffer_memory = index_buffer_memory;

    Ok(())
}

pub unsafe fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    data.uniform_buffers.clear();
    data.uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_buffers.push(uniform_buffer);
        data.uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok(())
}

pub unsafe fn update_uniform_buffer(app: &mut App, image_index: usize) -> Result<()> {
    let time = app.start.elapsed().as_secs_f32();

    let model = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), Deg(90.0) * time);

    let view = Mat4::look_at_rh(
        point3(2.0, 2.0, 2.0),
        point3(0.0, 0.0, 0.0),
        vec3(0.0, 0.0, 1.0),
    );

    let mut proj = cgmath::perspective(
        Deg(45.0),
        app.data.swapchain_extent.width as f32 / app.data.swapchain_extent.height as f32,
        0.1,
        10.0,
    );

    proj[1][1] *= -1.0; // Invert from OpenGL's Y system to Vulkan's

    let ubo = UniformBufferObject { model, view, proj };

    let memory = app.device.map_memory(
        app.data.uniform_buffers_memory[image_index],
        0,
        size_of::<UniformBufferObject>() as u64,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(&ubo, memory.cast(), 1);

    app.device
        .unmap_memory(app.data.uniform_buffers_memory[image_index]);

    Ok(())
}
