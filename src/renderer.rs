use std::{error::Error, time::Instant};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::vulkan::renderer::VulkanRenderer;

pub struct Renderer {
    vulkan: VulkanRenderer,
}

impl Renderer {
    pub fn new(
        window: &(impl HasDisplayHandle + HasWindowHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            vulkan: VulkanRenderer::new(window, width, height)?,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.vulkan.resize_requested = true;
        self.vulkan.resize_request_time = Instant::now();
        self.vulkan.width = width;
        self.vulkan.height = height;
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        self.vulkan.render()
    }

    pub fn run(&self) {
        self.vulkan.run();
    }
}
