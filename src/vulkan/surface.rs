use std::error::Error;

use ash::{Entry, Instance, khr::surface, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Surface {
    pub loader: surface::Instance,
    pub surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        window: &(impl HasWindowHandle + HasDisplayHandle),
    ) -> Result<Self, Box<dyn Error>> {
        let window_handle = window.window_handle()?.as_raw();
        let display_handle = window.display_handle()?.as_raw();

        let loader = surface::Instance::new(entry, instance);
        let surface = unsafe {
            ash_window::create_surface(entry, instance, display_handle, window_handle, None)?
        };
        log::info!("Surface created");

        Ok(Self { loader, surface })
    }

    pub fn destroy(&self) {
        unsafe { self.loader.destroy_surface(self.surface, None) };
    }
}
