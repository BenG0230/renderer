use std::{error::Error, ffi::CStr};

use ash::{Instance, vk};

use crate::vulkan::{surface::Surface, swapchain::SwapchainSupport};

pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    _graphics_queue: vk::Queue,
    _present_queue: vk::Queue,
}

fn get_required_device_extentions() -> [&'static CStr; 1] {
    [ash::khr::swapchain::NAME]
}

impl Device {
    pub fn new(instance: &Instance, surface: &Surface) -> Result<Self, Box<dyn Error>> {
        let physical_device = Self::pick_physical_device(instance, surface)?;
        let (device, graphics_queue, present_queue) =
            Self::create_logical_device(instance, surface, physical_device)?;

        Ok(Self {
            physical_device,
            device,
            _graphics_queue: graphics_queue,
            _present_queue: present_queue,
        })
    }

    fn pick_physical_device(
        instance: &Instance,
        surface: &Surface,
    ) -> Result<vk::PhysicalDevice, Box<dyn Error>> {
        // Choose first suitable device (also change this probably to rank system)
        let devices = unsafe { instance.enumerate_physical_devices()? };
        let device = devices
            .into_iter()
            .find(|d| Self::is_device_suitable(instance, surface, *d).unwrap());

        if device == None {
            return Err("No suitable physical devices found!".into());
        }

        // Log the name of the chosen device
        let props = unsafe { instance.get_physical_device_properties(device.unwrap()) };
        log::debug!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        Ok(device.unwrap())
    }

    fn is_device_suitable(
        instance: &Instance,
        surface: &Surface,
        device: vk::PhysicalDevice,
    ) -> Result<bool, Box<dyn Error>> {
        // Must have a queue that supports graphics and present
        // Must have support for all required extensions
        // Must have adequate swapchain support
        let (graphics, present) = Self::find_queue_families(instance, surface, device)?;
        let extension_support = Self::check_device_extension_support(instance, device)?;

        let is_swapchain_adequate = {
            let details = SwapchainSupport::new(device, surface)?;
            !details.formats.is_empty() && !details.present_modes.is_empty()
        };

        Ok(graphics.is_some() && present.is_some() && extension_support && is_swapchain_adequate)
    }

    fn check_device_extension_support(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<bool, Box<dyn Error>> {
        let required_extensions = get_required_device_extentions();

        let extension_props = unsafe { instance.enumerate_device_extension_properties(device)? };

        for required in required_extensions.iter() {
            let found = extension_props
                .iter()
                .any(|e| *required == e.extension_name_as_c_str().unwrap());

            if !found {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn find_queue_families(
        instance: &Instance,
        surface: &Surface,
        device: vk::PhysicalDevice,
    ) -> Result<(Option<u32>, Option<u32>), Box<dyn Error>> {
        // While usualy in the same family (0) they could be different
        // So must store them seperatly
        let mut graphics_index = None;
        let mut present_index = None;

        // Find a queue that supports graphics
        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (index, family) in props.iter().enumerate() {
            if family.queue_count <= 0 {
                continue;
            }

            // Check current family for graphics support
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics_index.is_none() {
                graphics_index = Some(index as u32);
            }

            // Check current family for presentation support
            let present_support = unsafe {
                surface.loader.get_physical_device_surface_support(
                    device,
                    index as u32,
                    surface.surface,
                )?
            };
            if present_support && present_index.is_none() {
                present_index = Some(index as u32);
            }

            if graphics_index.is_some() && present_index.is_some() {
                break;
            }
        }
        Ok((graphics_index, present_index))
    }

    fn create_logical_device(
        instance: &Instance,
        surface: &Surface,
        device: vk::PhysicalDevice,
    ) -> Result<(ash::Device, vk::Queue, vk::Queue), Box<dyn Error>> {
        // Safe to unwrap as physical device is chosen by now
        let (graphics_family_index, present_family_index) =
            Self::find_queue_families(instance, surface, device)?;
        let graphics_family_index = graphics_family_index.unwrap();
        let present_family_index = present_family_index.unwrap();

        // Priority needed even when only 1 queue is used
        let queue_priorities = [1.0f32];

        let queue_create_info = {
            let mut indices = vec![graphics_family_index, present_family_index];
            indices.dedup(); // Dont make the same queue twice!

            indices
                .iter()
                .map(|i| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(*i)
                        .queue_priorities(&queue_priorities)
                })
                .collect::<Vec<_>>()
        };

        let device_extensions = get_required_device_extentions();
        let device_extensions_ptrs = device_extensions
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_info)
            .enabled_extension_names(&device_extensions_ptrs)
            .enabled_features(&device_features);

        let device = unsafe { instance.create_device(device, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(graphics_family_index, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family_index, 0) };

        Ok((device, graphics_queue, present_queue))
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
