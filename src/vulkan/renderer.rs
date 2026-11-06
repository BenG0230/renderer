use std::error::Error;
use std::ffi::{CStr, CString, c_char, c_void};

use ash::{Entry, Instance, ext::debug_utils, khr, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle};

use crate::vulkan::debug::DebugMessenger;
use crate::vulkan::device::Device;
use crate::vulkan::surface::Surface;

pub(crate) struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,

    device: Device,
    surface: Surface,
    debug_messenger: Option<DebugMessenger>,
}

impl VulkanRenderer {
    pub fn new(window: &(impl HasDisplayHandle + HasWindowHandle)) -> Result<Self, Box<dyn Error>> {
        log::info!("Creating Vulkan Application!");

        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry, window.display_handle()?.as_raw())?;
        let debug_messenger = if super::debug::ENABLE_VALIDATION_LAYERS {
            Some(DebugMessenger::new(&entry, &instance)?)
        } else {
            None
        };

        let surface = Surface::new(&entry, &instance, window)?;
        let device = Device::new(&instance)?;

        Ok(Self {
            _entry: entry,
            instance,
            device,
            surface,
            debug_messenger,
        })
    }

    pub fn create_instance(
        entry: &Entry,
        display_handle: RawDisplayHandle,
    ) -> Result<Instance, Box<dyn Error>> {
        let app_name = CString::new("Playground").expect("Failed to build app_name CString");

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .api_version(vk::API_VERSION_1_3);

        let mut extension_names =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();
        if super::debug::ENABLE_VALIDATION_LAYERS {
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

        unsafe { Ok(entry.create_instance(&instance_create_info, None).unwrap()) }
    }

    pub fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const i8>) {
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
            // Must drop device and debug_messenger before the instance
            log::debug!("Dropping device");
            self.device.destroy();
            if let Some(debug_messenger) = &self.debug_messenger {
                log::debug!("Dropping debug messenger");
                debug_messenger.destroy();
            }
            log::debug!("Dropping vulkan application");
            self.instance.destroy_instance(None)
        };
    }
}
