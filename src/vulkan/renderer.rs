use std::{error::Error, ffi::CString};

use ash::{Entry, Instance, ext::debug_utils, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle};

use crate::vulkan::surface::Surface;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::{debug::DebugMessenger, device::Device};

pub(crate) struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,

    device: Device,

    swapchain: Swapchain,

    surface: Surface,
    debug_messenger: Option<DebugMessenger>,
}

impl VulkanRenderer {
    pub fn new(
        window: &(impl HasDisplayHandle + HasWindowHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Creating Vulkan Application!");

        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry, window.display_handle()?.as_raw())?;
        let debug_messenger = if super::debug::ENABLE_VALIDATION_LAYERS {
            Some(DebugMessenger::new(&entry, &instance)?)
        } else {
            None
        };

        let surface = Surface::new(&entry, &instance, window)?;
        let device = Device::new(&instance, &surface)?;

        let swapchain = Swapchain::new(&instance, &device, &surface, width, height)?;

        Ok(Self {
            _entry: entry,
            instance,
            device,
            swapchain,
            surface,
            debug_messenger,
        })
    }

    fn create_instance(
        entry: &Entry,
        display_handle: RawDisplayHandle,
    ) -> Result<Instance, Box<dyn Error>> {
        let app_name = CString::new("Playground").expect("Failed to build app_name CString");

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .api_version(vk::API_VERSION_1_3);

        // Generate required extentions based on window from window library
        // Chooses the correct ones for the system
        let mut extension_names =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();
        if super::debug::ENABLE_VALIDATION_LAYERS {
            // Enable debug extensions if used
            extension_names.push(debug_utils::NAME.as_ptr());
        }

        // Get names of wanted validation layers
        // must be done out here to appease the lifetime gods
        // layer_names must not be discarded as well
        let (_layer_names, layer_name_ptrs) = Self::get_layer_names_and_pointers();

        // Enable the required extensions in the instance
        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);
        if super::debug::ENABLE_VALIDATION_LAYERS {
            super::debug::check_validation_layer_support(entry)?;
            instance_create_info = instance_create_info.enabled_layer_names(&layer_name_ptrs);
        }

        unsafe { Ok(entry.create_instance(&instance_create_info, None)?) }
    }

    fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const i8>) {
        // Collect debug layer names in use, returns both the string and the pointers
        // to keep the pointers alive
        let layer_names = super::debug::REQUIRED_VALIDATION_LAYERS
            .iter()
            .map(|n| CString::new(*n).expect("Failed to build CString from layer name"))
            .collect::<Vec<_>>();
        let layer_name_ptrs = layer_names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();
        (layer_names, layer_name_ptrs)
    }

    pub fn run(&self) {
        log::info!("Running Application!");
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            // Wait for device to be idle before destroying everything
            log::debug!("Waiting for device to be idle");
            let result = self.device.device.device_wait_idle();
            if let Err(err) = result {
                log::error!("{:?}", err);
            }

            // Must drop in reverse order of creation
            log::debug!("Dropping swapchain");
            self.swapchain.destroy();

            log::debug!("Dropping device");
            self.device.destroy();

            log::debug!("Dropping surface");
            self.surface.destroy();

            if let Some(debug_messenger) = &self.debug_messenger {
                log::debug!("Dropping debug messenger");
                debug_messenger.destroy();
            }

            log::debug!("Dropping vulkan application");
            self.instance.destroy_instance(None)
        };
    }
}
