use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::Version;
use vulkano::instance::PhysicalDevice;

use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;

use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;

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

    let buffer = CpuAccessibleBuffer::from_data
        (
            device.clone(), // the device to create a buffer for (clones only the Arc) (does this increase the reference count? yes, it does).
            BufferUsage::all(), // the intended usage of the buffer, using the buffer in a way, which was not specified during construction will result in an error
            false, // is the Buffer CPU cached? false for most cases, only useful, if the cpu is expected to constantly stream data to the gpu by this
            data // content for the buffer
        ).expect("failed to create buffer");

    loop {

    }
}
