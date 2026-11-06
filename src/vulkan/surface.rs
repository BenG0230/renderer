use std::error::Error;

use ash::{Entry, Instance, khr::surface, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Surface {
    pub surface: ash::khr::surface::Instance,
    pub surface_khr: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        window: &(impl HasWindowHandle + HasDisplayHandle),
    ) -> Result<Self, Box<dyn Error>> {
        let window_handle = window.window_handle()?.as_raw();
        let display_handle = window.display_handle()?.as_raw();

        let surface = ash::khr::surface::Instance::new(entry, instance);

        Ok(Self {
            surface,
            surface_khr: unsafe {
                ash_window::create_surface(entry, instance, display_handle, window_handle, None)?
            },
        })
    }
}
