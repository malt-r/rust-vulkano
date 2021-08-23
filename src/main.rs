pub mod compute_test;
pub mod command_buffer_test;
pub mod image_test;
pub mod export_mandelbrot;
pub mod render_pass_sample;
pub mod window_test;

use vulkano::instance::Instance;
use vulkano::Version;
use vulkano::device::physical::PhysicalDevice;


use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};

// windowing
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::event::WindowEvent;
use winit::event::Event;

use winit::window::WindowBuilder;
use winit::event_loop::ControlFlow;
use winit::dpi::PhysicalSize;

use vulkano::swapchain::{AcquireError, SwapchainCreationError, Swapchain, SurfaceTransform, PresentMode, ColorSpace, FullscreenExclusive};
use vulkano::image::ImageUsage;

use vulkano::swapchain::Capabilities;
use vulkano::sync;
use vulkano::sync::{GpuFuture, FlushError};

use std::sync::Arc;
use vulkano::format::Format;

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

fn main() {
    println!("setting up device");

    // create instance
    let instance = {
        let extensions = vulkano_win::required_extensions(); // convenience method of vulkano_win to specify the required extensions for creation of a window on the current platform
        Instance::new
            (
                None,
                Version{major: 1, minor: 2, patch: 0},
                &extensions,
                None
            ).expect("Failed to create a new instance")
    };


    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");

    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }

    // find a queue of the physical device, which supports graphical operations
    let queue_family =
        physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    // device is actually an Arc<Device> (Atomically reference counted)
    let (device, mut queues) = {
        let device_extensions = DeviceExtensions { // require swapchain extension
            khr_swapchain: true,
            .. DeviceExtensions::none()
        };

        Device::new
            (
                physical, // the phsyical device
                &Features::none(), // requested features, which should be enabled on the device
                &device_extensions, // extensions
                [(queue_family, 0.5)].iter().cloned() // the Iterator on a list of queues, which should be used by this device with priorities
            ).expect("failed to create device")
    };

    // in this example code, the queues-Iterator only conains one element
    let queue = queues.next().unwrap();

    //command_buffer_test::execute(queue.clone(), device.clone());
    //compute_test::execute(queue.clone(), device.clone());
    //image_test::execute(queue.clone(), device.clone());
    //export_mandelbrot::execute(queue.clone(), device.clone());
    //render_pass_sample::execute(queue.clone(), device.clone());

    // create a new event loop
    let event_loop = EventLoop::new();

    // create a new 'surface', which refers to the object to draw on for the new window
    // this requires some vulkan extensions
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

    // query capabilities of the surface
    let caps = surface.capabilities(physical).unwrap();

    //s setup dimensions of the image, alpha behaviour and image format
    let dimensions = caps.current_extent.unwrap_or([1280, 1024]);
    let alpha = caps.supported_composite_alpha.iter().next().unwrap(); // just use the second one?
    let format = caps.supported_formats[0].0; // just use the first one?

    let swapchain_builder = Swapchain::start(device.clone(), surface.clone());
    let (swapchain, images) = swapchain_builder
        .num_images(caps.min_image_count)
        .format(format)
        .dimensions(dimensions)
        .usage(ImageUsage::color_attachment())
        .transform(SurfaceTransform::Identity)
        .composite_alpha(alpha)
        .present_mode(PresentMode::Fifo)
        .fullscreen_exclusive(FullscreenExclusive::Default)
        .clipped(true)
        .color_space(ColorSpace::SrgbNonLinear)
        .layers(1)
        .build()
        .expect("Swapchain creation failed");


    // load shader for device
    let vertex_shader = vs::Shader::load(device.clone()).expect("failed to create vertex shader");
    let fragment_shader = fs::Shader::load(device.clone()).expect("failed to create fragment shader");

    // specify a simple renderpass with a color attachment and only one pass
    // the color-attachment is passed to the color slot of the pass-struct
    //
    // a renderpass describes, where the output of a pipeline will go (to which attachment)
    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: { // declare color as an attachement to the render pass
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color], // use the color-attachment as an input to the pass
            depth_stencil: {}
        }
    ).unwrap());


    // TODO: create Vertex buffer and pipeline and state and framebuffer

    // match event to windowEvent::CloseRequested and set the control_flow to
    // Exit afterward
    //
    // also wire up the window-resize event to print changed window dimensions
    let mut recreate_swapchain = false;
    let mut previous_end_frame = Some(sync::now(device.clone()).boxed());
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(phys_size),
                ..
            } => {
                println!("new width:{}, new height:{}", phys_size.width, phys_size.height);
            },
            Event::RedrawEventsCleared => {
                // TODO: handle resizing (recreating the swapchain)
                // this will be the main render loop, i guess
                //
                // returns the index of the acquired image, a boolean, which indicates
                // whether the acquired image is still usable but suboptimal (sc should be
                // recreated) and an acquisition future..
                //
                // the acquisition future will be signaled, if the image is ready for
                // drawing to it
                let (image_num, suboptimal, acquire_future) =
                    match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(result) => result,
                    Err(AcquireError::OutOfDate) => { // swapchain was made unusable by some change and needs to be recreated
                        recreate_swapchain = true;
                        return;
                    },
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

                // create command buffer
                let mut builder = AutoCommandBufferBuilder::primary
                    (
                        device.clone(),
                        queue_family,
                        CommandBufferUsage::OneTimeSubmit
                    ).unwrap();



                // execute command_buffer and present image
                let future = previous_end_frame
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    // after the acquisition future is signaled, the image can be drawn to.
                    // this creates a presentation future.
                    //
                    // apperently, this future holds the resources 'for as long as they are used
                    // by the GPU'
                    // destroying the future will block until the GPU is finished with
                    // the submitted command buffer.
                    .unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_end_frame = Some(future.boxed());
                    },
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_end_frame = Some(sync::now(device.clone()).boxed());
                    },
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_end_frame = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _=>()
        }
    });
}

