use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;

// atomically reference counted
use std::sync::Arc;
use vulkano::pipeline::ComputePipeline;
//use vulkano::pipeline::layout::PipelineLayout;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::pipeline::ComputePipelineAbstract;

//use simple_stopwatch::Stopwatch;

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

pub fn execute(queue: Arc<Queue>, device: Arc<Device>) {
    println!("performing compute test");

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
    println!("Compute operation successful!");
}
