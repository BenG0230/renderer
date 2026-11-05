use std::error::Error;

use crate::vulkan::renderer::VulkanRenderer;

pub struct Renderer {
    vulkan: VulkanRenderer,
}

impl Renderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            vulkan: VulkanRenderer::new()?,
        })
    }

    pub fn run(&self) {
        self.vulkan.run();
    }
}
