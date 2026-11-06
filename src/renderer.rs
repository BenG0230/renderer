use std::error::Error;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::vulkan::renderer::VulkanRenderer;

pub struct Renderer {
    vulkan: VulkanRenderer,
}

impl Renderer {
    pub fn new(window: &(impl HasDisplayHandle + HasWindowHandle)) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            vulkan: VulkanRenderer::new(window)?,
        })
    }

    pub fn lick() {
        log::error!("Lick!");
    }

    pub fn run(&self) {
        self.vulkan.run();
    }
}
