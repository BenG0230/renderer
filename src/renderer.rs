use std::error::Error;

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

    pub fn lick() {
        log::error!("Lick!");
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        self.vulkan.render()
    }

    pub fn run(&self) {
        self.vulkan.run();
    }
}
