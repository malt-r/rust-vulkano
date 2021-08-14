use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;

// atomically reference counted
use std::sync::Arc;

use simple_stopwatch::Stopwatch;

pub fn execute(queue: Arc<Queue>, device: Arc<Device>) {
    println!("performing command buffer test");
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
    let mut builder = AutoCommandBufferBuilder::primary
        (
            device.clone(),
            queue.family(),
            CommandBufferUsage::SimultaneousUse
        ).unwrap();

    builder.copy_buffer(input_buffer.clone(), output_buffer.clone()).unwrap();
    let command_buffer = builder.build().unwrap();

    let sw = Stopwatch::start_new();

    // needs to be submitted and synched
    let finished = command_buffer.execute(queue.clone()).unwrap();

    // ??? is there even lsp suppor for this?
    finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

    // read from buffers
    let input_content = input_buffer.read().unwrap();
    let output_content = output_buffer.read().unwrap();

    let us = sw.us();

    debug_assert_eq!(&*input_content, &*output_content, "input content is not equal to output content");
    println!("timed {} us", us);
}
