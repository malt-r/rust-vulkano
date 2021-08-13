use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::Version;
use vulkano::device::physical::PhysicalDevice;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::sync::GpuFuture;

// atomically reference counted
use std::sync::Arc;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::layout::PipelineLayout;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::ComputePipelineAbstract;

use simple_stopwatch::Stopwatch;

mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
    }
}

fn main() {
    println!("Hello, world!");
    let instance = Instance::new
        (
            None,
            Version
            {
                major: 1,
                minor: 2,
                patch: 0
            },
            &InstanceExtensions::none(),
            None
        )
        .expect("failed to create instance");

    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");

    for family in physical.queue_families() {
        println!("Found a queue family with {:?} queue(s)", family.queues_count());
    }

    // find a queue of the physical device, which supports graphical operations
    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    // device is actually an Arc<Device> (Atomically reference counted)
    let (device, mut queues) = {
        Device::new
            (
                physical, // the phsyical device
                &Features::none(), // requested features, which should be enabled on the device
                &DeviceExtensions::none(), // extensions
                [(queue_family, 0.5)].iter().cloned() // the Iterator on a list of queues, which should be used by this device with priorities
            ).expect("failed to create device")
    };

    // in this example code, the queues-Iterator only conains one element
    let queue = queues.next().unwrap();

    let data = 12; // sample data

    let _sample_buffer = CpuAccessibleBuffer::from_data
        (
            device.clone(), // the device to create a buffer for (clones only the Arc) (does this increase the reference count? yes, it does).
            BufferUsage::all(), // the intended usage of the buffer, using the buffer in a way, which was not specified during construction will result in an error
            false, // is the Buffer CPU cached? false for most cases, only useful, if the cpu is expected to constantly stream data to the gpu by this
            data // content for the buffer
        ).expect("failed to create buffer");

    // create input data as range from 0 to 63 and output as 64 zeros
    let input_data = 0..63;
    let output_data = (0..63).map(|_|0);

    let input_buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(), // the device to create a buffer for (clones only the Arc) (does this increase the reference count? yes, it does).
            BufferUsage::all(), // the intended usage of the buffer, using the buffer in a way, which was not specified during construction will result in an error
            false, // is the Buffer CPU cached? false for most cases, only useful, if the cpu is expected to constantly stream data to the gpu by this
            input_data // content for the buffer
        ).expect("failed to create buffer");

    let output_buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(), // the device to create a buffer for (clones only the Arc) (does this increase the reference count? yes, it does).
            BufferUsage::all(), // the intended usage of the buffer, using the buffer in a way, which was not specified during construction will result in an error
            false, // is the Buffer CPU cached? false for most cases, only useful, if the cpu is expected to constantly stream data to the gpu by this
            output_data // content for the buffer
        ).expect("failed to create buffer");

    // sample command buffer
    //let mut builder = AutoCommandBufferBuilder::primary
    //    (
    //        device.clone(),
    //        queue.family(),
    //        CommandBufferUsage::SimultaneousUse
    //    ).unwrap();

    //builder.copy_buffer(input_buffer.clone(), output_buffer.clone()).unwrap();
    //let command_buffer = builder.build().unwrap();

    //let sw = Stopwatch::start_new();

    //// needs to be submitted and synched
    //let finished = command_buffer.execute(queue.clone()).unwrap();

    //// ??? is there even lsp suppor for this?
    //finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    //// read from buffers
    //let input_content = input_buffer.read().unwrap();
    //let output_content = output_buffer.read().unwrap();

    //let us = sw.us();

    //debug_assert_eq!(&*input_content, &*output_content, "input content is not equal to output content");
    //println!("timed {} us", us);

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

    // create buffer
    let iter  = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter
        (
            device.clone(),
            BufferUsage::all(),
            false,
            iter
        ).expect("Error creating data buffer");

    // bind buffer to pipeline
    let layout_slice = compute_pipeline.layout().descriptor_set_layouts();
    let layout = &layout_slice[0];

    // create new descriptor_set and add buffer at position 0
    // Vulkan requires supply of a pipeline for the creation of a descriptor set
    // but this set can be used with other pipelines afterward, so long as the
    // layout matches the expected layout by the shader
    let set = Arc::new
        (PersistentDescriptorSet::start
         (
             layout.clone()
         )
         .add_buffer
         (
             data_buffer.clone()
         )
         .unwrap()
         .build()
         .unwrap());

    let mut builder = AutoCommandBufferBuilder::primary(device.clone(), queue.family(), CommandBufferUsage::SimultaneousUse).unwrap();
    builder.dispatch
        (
            [1024,1,1],
            compute_pipeline.clone(),
            set.clone(),
            ()
        ).unwrap();
    let command_buffer = builder.build().unwrap();
    let finished = command_buffer.execute(queue.clone()).unwrap();

    // wait for completion of operation
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    // check result
    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }
    println!("Operation complete!");
}
