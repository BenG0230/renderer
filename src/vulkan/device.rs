use std::{error::Error, ffi::CStr};

use ash::{Instance, vk};

pub struct Device {
    _physical_device: vk::PhysicalDevice,
}

impl Device {
    pub fn new(instance: &Instance) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            _physical_device: Self::pick_physical_device(instance)?,
        })
    }

    fn pick_physical_device(instance: &Instance) -> Result<vk::PhysicalDevice, Box<dyn Error>> {
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
        // Find a queue that supports graphics
        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };
        props
            .iter()
            .enumerate()
            .find(|(_, family)| {
                family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(index, _)| index as u32)
    }
}
