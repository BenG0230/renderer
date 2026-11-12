use std::error::Error;

use ash::{Instance, khr::swapchain, vk};

use crate::vulkan::{device::Device, surface::Surface};

pub struct Swapchain {
    pub loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    _image_format: vk::Format,
    _extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
}

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
        let image_count = {
            // Want min image count + 1 as sticking to the minimum results in the program sometimes
            // waiting for the driver to finish
            let mut image_count = support.capabilities.min_image_count + 1;
            // Max image count of 0 means unlimited images available
            if support.capabilities.max_image_count > 0
                && support.capabilities.max_image_count < image_count
            {
                image_count = support.capabilities.max_image_count;
            }
            image_count
        };

        log::debug!(
            "Creating swapchain:\n\tFormat: {:?}\n\tColour space: {:?}\n\tPresent mode: {:?}\n\tExtent: {:?}\n\tImage count: {}",
            format.format,
            format.color_space,
            present_mode,
            extent,
            image_count
        );

        let family_indices = [device.graphics_queue_index, device.present_queue_index];

        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

        if device.graphics_queue_index != device.graphics_queue_index {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&family_indices);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        create_info = create_info
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let loader = swapchain::Device::new(instance, &device.device);
        let swapchain = unsafe { loader.create_swapchain(&create_info, None)? };
        let images = unsafe { loader.get_swapchain_images(swapchain)? };

        let image_views = Self::create_image_views(device, &images, format.format)?;

        Ok(Self {
            loader,
            swapchain,
            images,
            _image_format: format.format,
            _extent: extent,
            image_views,
        })
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
            // The extent can be set by us only if the current extent is u32 Max
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

    fn create_image_views(
        device: &Device,
        images: &[vk::Image],
        image_format: vk::Format,
    ) -> Result<Vec<vk::ImageView>, Box<dyn Error>> {
        let mut image_views = Vec::new();
        for image in images {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(image_format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = unsafe { device.device.create_image_view(&create_info, None)? };
            image_views.push(image_view);
        }

        Ok(image_views)
    }

    pub fn destroy(&self, device: &Device) {
        log::debug!("Dropping swapchain image views");
        for image_view in &self.image_views {
            unsafe { device.device.destroy_image_view(*image_view, None) };
        }

        log::debug!("Dropping swapchain");
        unsafe { self.loader.destroy_swapchain(self.swapchain, None) };
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
