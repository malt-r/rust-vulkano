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

use vulkano::pipeline::GraphicsPipeline;

use vulkano::render_pass::Subpass;
use vulkano::command_buffer::SubpassContents;
use vulkano::render_pass::Framebuffer;
use vulkano::sync::GpuFuture;
use vulkano::command_buffer::DynamicState;
use vulkano::pipeline::viewport::Viewport;

// atomically reference counted
use std::sync::Arc;

use image::{Rgba, ImageBuffer};

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location=0) in vec2 position;
void main() {
    // gl_Position needs to be set, as it is the global 'variable', which determines
    // the pixels position in framebuffer space
    gl_Position = vec4(position, 0.0, 1.0);
}
"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
#version 450

layout(location=0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
    }
}

#[derive(Default, Copy, Clone)]
struct Vertex {
    position: [f32;2],
}

vulkano::impl_vertex!(Vertex, position);

pub fn execute(queue: Arc<Queue>, device: Arc<Device>) {
    // load shader for device
    let vertex_shader = vs::Shader::load(device.clone()).expect("failed to create vertex shader");
    let fragment_shader = fs::Shader::load(device.clone()).expect("failed to create fragment shader");

    // specify a simple renderpass with a color attachment and only one pass
    // the color-attachment is passed to the color slot of the pass-struct
    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: { // declare color as an attachement to the render pass
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8Unorm,
                samples: 1,
            }
        },
        pass: {
            color: [color], // use the color-attachment as an input to the pass
            depth_stencil: {}
        }
    ).unwrap());

    // create 1024x1024 image object
    let image = StorageImage::new
        (
            device.clone(),
            ImageDimensions::Dim2d{width: 1024, height: 1024, array_layers: 1},
            Format::R8G8B8A8Unorm,
            Some(queue.family())
        ).unwrap();

    // create CpuAccessibleBuffer to copy the image to after rendering
    let buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(),
            BufferUsage::all(),
            false,
            (0..1024 * 1024 * 4).map(|_|0u8) // 1024 x 1024
        ).expect("Failed to create buffer");

    // create view for iamge, seems to be required to setup a framebuffer
    let image_view = ImageView::new(image.clone()).unwrap();

    // set up the framebuffer and attach the image as the color-attachment
    // the attachments need to be attached to the framebuffer in the same order
    // as to the render_pass
    let framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
                               .add(image_view.clone()).unwrap()
                               .build().unwrap());

    // create render pipeline

    let pipeline =
        Arc::new (
            GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()                         // set the input to a single vertex buffer and specify template argument
            .vertex_shader(vertex_shader.main_entry_point(), ())            // pass the compiled vertex shader
            .fragment_shader(fragment_shader.main_entry_point(), ())        // pass the compiled fragment shader
            .viewports_dynamic_scissors_irrelevant(1)                       // set the viewport scissor boxes (which determine, what will be drawn) to cover the whole viewport
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())    // pass the configured render pass
            .build(device.clone())                                          // build for device
            .unwrap()
            );

    // create dynamic viewport state (don't really understand, what this is doing..)
    // The state of a pipeline can't really be modified, once it is created.
    // When using dynamic state, some parts of the pipeline state can be modified
    // (in the following this is restricted to the viewport)
    let dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [1024.0, 1024.0],
            depth_range: 0.0..1.0,
        }]),
        .. DynamicState::none()
    };

    // create triangle veritces
    let vertex1 = Vertex { position: [-0.5, -0.5]};
    let vertex2 = Vertex { position: [ 0.0,  0.5]};
    let vertex3 = Vertex { position: [ 0.5, -0.25]};

    // create vertex buffer from vertices
    let vertex_buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(),
            BufferUsage::all(),
            false,
            vec![vertex1, vertex2, vertex3].into_iter()
        ).unwrap();

    // build command buffer
    let mut builder = AutoCommandBufferBuilder::primary
        (
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

    builder
        .begin_render_pass
        (
            framebuffer.clone(),    // the framebuffer to render to
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()] // the clear value
        ).unwrap()

        .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ())
        .unwrap()

        .end_render_pass()
        .unwrap()

        .copy_image_to_buffer(image.clone(), buffer.clone())
        .unwrap();


    // build command from builder and execute
    let command = builder.build().unwrap();

    // execute command buffer
    let finished = command.execute(queue.clone()).unwrap();
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    // convert result to image
    let buffer_content = buffer.read().unwrap();
    let image_buffer = ImageBuffer::<Rgba<u8>,_>::from_raw(1024, 1024, &buffer_content[..]).unwrap();

    image_buffer.save("triangle.png").unwrap();

}
