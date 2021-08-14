use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::Queue;

use vulkano::image::ImageDimensions;
use vulkano::image::StorageImage;
use vulkano::image::view::ImageView;
use vulkano::format::Format;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::ComputePipelineAbstract;
use vulkano::sync::GpuFuture;

// atomically reference counted
use std::sync::Arc;

use image::{Rgba, ImageBuffer};

mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: "
#version 450

// use 8x8 group size for better parallelism
// will use GlobalInvocation index for identification of pixel to write to
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in ;

// rgba defines the format of the image buffer
layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

void main() {
    // This aims to transform the globalInvocation ID from invocation Index (0..size)
    // (starting at the top left corner) to a complex number c with x in [-2,0] and
    // y [-1, 1], I think.
    // Tutorial version:
    // where is this first + vec(0.5) coming from? Is this to ensure proper integer division?
    // vec2 normalized_coords = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    // vec2 c = (normalized_coords - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
    //
    vec2 normalized_coords = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
    vec2 c = (normalized_coords - vec2(0.5)) * 2.0 - vec2(0.5, 0.0);

    // actual implementation of the mandelbrot check
    vec2 z = vec2(0.0, 0.0);
    float i;

    for (i = 0.0; i < 1.0; i += 0.005) {
        z = vec2(
            z.x*z.x - z.y*z.y + c.x,
            z.x*z.y + z.x*z.y + c.y
        );

        // length is a built-in GLSL function
        if (length(z) > 4.0) {
            break;
        }
    }

    // store the value as greyscale value in pixel at coordinate indicated by invocation index
    vec4 toWrite = vec4(vec3(i), 1.0);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), toWrite);
}
"
    }
}

pub fn execute(queue: Arc<Queue>, device: Arc<Device>) {

    // TODO: build GPU-side image buffer -
    // TODO: build CPU-side image buffer -
    // TODO: bind GPU-side image buffer to pipeline (via descriptor)
    // TODO: create command buffer
    // TODO: dispatch command

    // load shader for device
    let shader = cs::Shader::load(device.clone()).expect("failed to create compute pipeline");

    // needs device, entry point for shader and specs constants (empty)
    let compute_pipeline = Arc::new
        (
            ComputePipeline::new
            (
                device.clone(),
                &shader.main_entry_point(),
                &(),
                None
            ).expect("Error to create compute pipeline")
        );

    // bind buffer to pipeline
    let layout_slice = compute_pipeline.layout().descriptor_set_layouts();
    let layout = &layout_slice[0];

    // create image
    let image = StorageImage::new
        (
            device.clone(),
            ImageDimensions::Dim2d{width: 1024, height: 1024, array_layers: 1},
            Format::R8G8B8A8Unorm,
            Some(queue.family())
        ).unwrap();
    let image_view = ImageView::new(image.clone()).unwrap();

    // create new descriptor_set and add buffer at position 0
    // Vulkan requires supply of a pipeline for the creation of a descriptor set
    // but this set can be used with other pipelines afterward, so long as the
    // layout matches the expected layout by the shader
    let set = Arc::new
        (PersistentDescriptorSet::start
         (
             layout.clone()
         )
         .add_image
         (
             image_view
         )
         .unwrap()
         .build()
         .unwrap());


    // build command buffer
    let mut builder = AutoCommandBufferBuilder::primary
        (
            device.clone(),
            queue.family(),
            CommandBufferUsage::SimultaneousUse
        ).unwrap();

    // create CpuAccessibleBuffer to copy the image to
    let buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(),
            BufferUsage::all(),
            false,
            (0..1024 * 1024 * 4).map(|_|0u8) // 1024 x 1024
        ).expect("Failed to create buffer");

    builder
        .dispatch([1024 / 8, 1024 / 8, 1], compute_pipeline.clone(), set.clone(), ()).unwrap()
        .copy_image_to_buffer(image.clone(), buffer.clone()).unwrap();

    let command = builder.build().unwrap();

    // execute command buffer
    let finished = command.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    // convert result to image
    let buffer_content = buffer.read().unwrap();
    let image_buffer = ImageBuffer::<Rgba<u8>,_>::from_raw(1024, 1024, &buffer_content[..]).unwrap();

    image_buffer.save("mandelbrot.png").unwrap();

}
