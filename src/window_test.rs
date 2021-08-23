use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::event::WindowEvent;
use winit::event::Event;
use winit::window::WindowBuilder;
use winit::event_loop::ControlFlow;

use vulkano::instance::Instance;
use vulkano::device::Device;
use vulkano::device::Queue;


// atomically reference counted
use std::sync::Arc;

pub fn execute(queue: Arc<Queue>, device: Arc<Device>, instance: Arc<Instance>) {
    // create a new event loop
    let event_loop = EventLoop::new();

    // create a new 'surface', which refers to the object to draw on for the new window
    // this requires some vulkan extensions
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

    // match event to windowEvent::CloseRequested and set the control_flow to
    // Exit afterward
    event_loop.run(|event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                *control_flow = ControlFlow::Exit
            },
            _=>()
        }
    });

}
