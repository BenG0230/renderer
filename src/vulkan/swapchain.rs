use std::{error::Error, thread::available_parallelism};

use ash::{Instance, vk};

use crate::vulkan::{device::Device, surface::Surface};

pub struct Swapchain {}

impl Swapchain {
    pub fn new(
        instance: &Instance,
        device: &Device,
        surface: &Surface,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let support = SwapchainSupport::new(device.physical_device, surface)?;
        let format = Self::choose_swapchain_surface_format(&support.formats);
        let present_mode = Self::choose_swapchain_present_mode(&support.present_modes);
        let extent = Self::choose_swapchain_extent(&support.capabilities, width, height);

        Ok(Self {})
    }

    fn choose_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        // Should always have a R8G8B8A8_SRGB and SRGB_NONLINEAR format (says how to vulkan 2026)
        *formats
            .iter()
            .find(|f| {
                f.format == vk::Format::R8G8B8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&formats[0])
    }

    fn choose_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        // IMMEDIATE - Images submitted are transferred to the screen right away (screen tearing)
        // FIFO - Images are put into a queue, if the queue is full the program has to wait
        //        only displays images on vblan, similar to VSync
        // FIFO Relaxed - If the queue is empty when refreshing the display, instead of waiting for
        //        next vblank when image is submitted display immediately
        // MAILBOX - If the queue is full, replace images in the queue instead of blocking the
        //        program (triple buffering)

        if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            // Seems like the one everyone is using so 🤷‍♂️
            vk::PresentModeKHR::MAILBOX
        } else {
            // Guaranteed to be available, so best fallback
            vk::PresentModeKHR::FIFO
        }
    }

    fn choose_swapchain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: u32,
        height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

pub struct SwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn new(device: vk::PhysicalDevice, surface: &Surface) -> Result<Self, Box<dyn Error>> {
        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device, surface.surface)
        }?;

        let formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device, surface.surface)
        }?;

        let present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device, surface.surface)
        }?;

        Ok(Self {
            capabilities,
            formats,
            present_modes,
        })
    }
}
