pub mod compute_test;
pub mod command_buffer_test;
pub mod image_test;
pub mod export_mandelbrot;
pub mod render_pass_sample;
pub mod window_test;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::DynamicState;
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::command_buffer::SubpassContents;
use vulkano::command_buffer::pool::CommandPoolBuilderAlloc;
use vulkano::image::SwapchainImage;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::Version;
//use vulkano::instance::PhysicalDevice;
use vulkano::device::physical::PhysicalDevice;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::render_pass::FramebufferAbstract;
use vulkano::render_pass::Framebuffer;
use vulkano::render_pass::Subpass;
use vulkano::render_pass::RenderPass;
use vulkano::command_buffer::pool::CommandPool;
use vulkano::image::view;


use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};

// windowing
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::event::WindowEvent;
use winit::event::Event;

use winit::window::Window;
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

//mod vs {
//    vulkano_shaders::shader!{
//        ty: "vertex",
//        src: "
//#version 450
//
//layout(location=0) in vec2 position;
//void main() {
//    // gl_Position needs to be set, as it is the global 'variable', which determines
//    // the pixels position in framebuffer space
//    gl_Position = vec4(position, 0.0, 1.0);
//}
//"
//    }
//}
//
//mod fs {
//    vulkano_shaders::shader!{
//        ty: "fragment",
//        src: "
//#version 450
//
//layout(location=0) out vec4 f_color;
//
//void main() {
//    f_color = vec4(1.0, 0.0, 0.0, 1.0);
//}
//"
//    }
//}
//
//#[derive(Default, Copy, Clone)]
//struct Vertex {
//    position: [f32;2],
//}
//
//vulkano::impl_vertex!(Vertex, position);
//


fn main() {
    println!("setting up device");
    // The first step of any Vulkan program is to create an instance.
    //
    // When we create an instance, we have to pass a list of extensions that we want to enable.
    //
    // All the window-drawing functionalities are part of non-core extensions that we need
    // to enable manually. To do so, we ask the `vulkano_win` crate for the list of extensions
    // required to draw to a window.
    let required_extensions = vulkano_win::required_extensions();

    // Now creating the instance.
    let instance = Instance::new(None, Version::V1_2, &required_extensions, None).unwrap();

    {
        // We then choose which physical device to use.
        //
        // In a real application, there are three things to take into consideration:
        //
        // - Some devices may not support some of the optional features that may be required by your
        //   application. You should filter out the devices that don't support your app.
        //
        // - Not all devices can draw to a certain surface. Once you create your window, you have to
        //   choose a device that is capable of drawing to it.
        //
        // - You probably want to leave the choice between the remaining devices to the user.
        //
        // For the sake of the example we are just going to use the first device, which should work
        // most of the time.

        // TODO: fix reference lifetime, why it this happening?! some change in
        // in the source of enumerate?
        //let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
        //let instance = Instance::new(None, Version::V1_1, &InstanceExtensions::none(), None).unwrap();
        let physical = PhysicalDevice::from_index(&instance, 0).unwrap();
        //let physical = PhysicalDevice::from_index(&instance, 0).unwrap();

        // create a new event loop
        let event_loop = EventLoop::new();

        // create a new 'surface', which refers to the object to draw on for the new window
        // this requires some vulkan extensions
        let surface = WindowBuilder::new().build_vk_surface
            (
                &event_loop,
                instance.clone()
            ).unwrap();

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

        #[derive(Default, Debug, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        vulkano::impl_vertex!(Vertex, position);

        // We now create a buffer that will store the shape of our triangle.
        let vertex_buffer = {

            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-0.5, -0.25],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                    },
                    Vertex {
                        position: [0.25, -0.1],
                    },
                ]
                .iter()
                .cloned(),
            )
            .unwrap()
        };

        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
                    #version 450
                    layout(location = 0) in vec2 position;
                    void main() {
                        gl_Position = vec4(position, 0.0, 1.0);
                    }
                "
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
                    #version 450
                    layout(location = 0) out vec4 f_color;
                    void main() {
                        f_color = vec4(1.0, 0.0, 0.0, 1.0);
                    }
                "
            }
        }

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


        // TODO: create dynamic state and framebuffer
        let mut dynamic_state = DynamicState {
            line_width: None,
            viewports: None,
            scissors: None,
            compare_mask: None,
            write_mask: None,
            reference: None,
        };

        let mut framebuffers = window_size_dependent_setup
            (
                &images,
                render_pass.clone(),
                &mut dynamic_state
            );

        // create triangle veritces
        //let vertex1 = Vertex { position: [-0.5, -0.5]};
        //let vertex2 = Vertex { position: [ 0.0,  0.5]};
        //let vertex3 = Vertex { position: [ 0.5, -0.25]};

        //// create vertex buffer from vertices
        //let vertex_buffer = CpuAccessibleBuffer::from_iter
        //    (
        //        device.clone(),
        //        BufferUsage::all(),
        //        false,
        //        vec![vertex1, vertex2, vertex3].into_iter()
        //    ).unwrap();

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

        // Event loop
        //
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

                    // should be called time to time, to free up ressources and locks
                    // used by the gpu associated with this GPU Future.
                    previous_end_frame.as_mut().unwrap().cleanup_finished();

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

                    let clear_values = vec![[1.0, 1.0, 1.0, 1.0].into()];

                    // TODO: create command buffer
                    builder
                        .begin_render_pass
                        (
                            framebuffers[image_num].clone(),
                            SubpassContents::Inline,
                            clear_values
                        ).unwrap()
                        .draw
                        (
                            pipeline.clone(),
                            &dynamic_state,
                            vertex_buffer.clone(),
                            (),
                            (),
                        ).unwrap();

                    let command_buffer = builder.build().unwrap();

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
                        },
                    }
                }
                _=>(),
            }
        });
    }

}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> { // this notation specifies bounds on the implemented traits of the template parameter, i guess
    let dimensions = images[0].dimensions();

    // setup viewport data
    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    // create a dynamic state with the viewport specification
    dynamic_state.viewports = Some(vec![viewport]);

    // create a framebuffer for each image
    images
        .iter()
        .map(|image| {
            // map allows for a function to capture the element of
            // the iterator and perform some calculation/tranformation on it
            let image_view = view::ImageView::new(image.clone()).unwrap();
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image_view.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
