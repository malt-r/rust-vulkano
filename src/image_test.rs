use vulkano::device::Device;
use vulkano::device::Queue;

use vulkano::image::ImageDimensions;
use vulkano::image::StorageImage;
use vulkano::format::Format;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::sync::GpuFuture;

// atomically reference counted
use std::sync::Arc;

use image::{Rgba, ImageBuffer};

pub fn execute(queue: Arc<Queue>, device: Arc<Device>) {

    // create image
    let image = StorageImage::new
        (
            device.clone(),
            ImageDimensions::Dim2d{width: 1024, height: 1024, array_layers: 1},
            Format::R8G8B8A8Unorm,
            Some(queue.family())
        ).unwrap();

    // create CpuAccessibleBuffer to copy the image to
    let buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(),
            BufferUsage::all(),
            false,
            (0..1024 * 1024 * 4).map(|_|0u8) // 1024 x 1024
        ).expect("Failed to create buffer");

    // build command buffer
    let mut command_buffer_builder = AutoCommandBufferBuilder::primary
        (
            device.clone(),
            queue.family(),
            CommandBufferUsage::SimultaneousUse
        ).unwrap();

    command_buffer_builder
        .clear_color_image
            (
                image.clone(),
                vulkano::format::ClearValue::Float([0.0, 0.0, 1.0, 1.0]) // RGBA
            ).unwrap()
        .copy_image_to_buffer
            (
                image.clone(), // source
                buffer.clone() // dest
            ).unwrap();
    let command = command_buffer_builder.build().unwrap();

    // execute command buffer
    let finished = command.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    // convert result to image
    let buffer_content = buffer.read().unwrap();
    let image_buffer = ImageBuffer::<Rgba<u8>,_>::from_raw(1024, 1024, &buffer_content[..]).unwrap();

    image_buffer.save("image.png").unwrap();

}

