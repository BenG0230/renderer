use std::{error::Error, ffi::CStr};

use ash::{Instance, vk};

pub struct Device {
    _physical_device: vk::PhysicalDevice,
    device: ash::Device,
    _graphics_queue: vk::Queue,
}

impl Device {
    pub fn new(instance: &Instance) -> Result<Self, Box<dyn Error>> {
        let physical_device = Self::pick_physical_device(instance)?;
        let (device, graphics_queue) = Self::create_logical_device(instance, physical_device)?;

        Ok(Self {
            _physical_device: physical_device,
            device,
            _graphics_queue: graphics_queue,
        })
    }

    fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice, Box<dyn Error>> {
        // Choose first suitable device (also change this probably to rank system)
        let devices = unsafe { instance.enumerate_physical_devices()? };
        let device = devices
            .into_iter()
            .find(|d| Self::is_device_suitable(instance, *d));

        if device == None {
            return Err("No suitable physical devices found!".into());
        }

        let props = unsafe { instance.get_physical_device_properties(device.unwrap()) };
        log::debug!("Selected physical device: {:?}", unsafe {
            CStr::from_ptr(props.device_name.as_ptr())
        });

        Ok(device.unwrap())
    }

    fn is_device_suitable(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        Self::find_queue_families(instance, device).is_some()
    }

    fn find_queue_families(instance: &Instance, device: vk::PhysicalDevice) -> Option<u32> {
        let mut graphics_index = None;

        // Find a queue that supports graphics
        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (index, family) in props.iter().enumerate() {
            if family.queue_count <= 0 {
                continue;
            }

            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) && graphics_index.is_none() {
                graphics_index = Some(index as u32);
            }

            if graphics_index.is_some() {
                break;
            }
        }

        graphics_index
    }

    fn create_logical_device(
        instance: &Instance,
        device: vk::PhysicalDevice,
    ) -> Result<(ash::Device, vk::Queue), Box<dyn Error>> {
        // Safe to unwrap as physical device is chosen by now
        let queue_family_index = Self::find_queue_families(instance, device).unwrap();
        // Priority needed even when only 1 queue is used
        let queue_priorities = [1.0f32];

        let queue_create_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)];

        // Nothing fancy needed yet
        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_info)
            .enabled_features(&device_features);

        let device = unsafe { instance.create_device(device, &device_create_info, None)? };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok((device, graphics_queue))
    }

    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
