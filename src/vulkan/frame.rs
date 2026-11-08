use ash::vk;

pub struct Frame {
    command_buffer: vk::CommandBuffer,
    image_available: vk::Semaphore,
    in_flight_fence: vk::Fence,
}
