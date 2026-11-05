use std::error::Error;
use std::ffi::{CStr, CString, c_char, c_void};

use ash::{Entry, Instance, ext::debug_utils, khr, vk};

use crate::vulkan::debug::DebugMessenger;
use crate::vulkan::device::Device;

const ENABLE_VALIDATION_LAYERS: bool = true;
const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub(crate) struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,

    device: Device,
    debug_messenger: Option<DebugMessenger>,
}

impl VulkanRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        log::info!("Creating Vulkan Application!");

        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry)?;
        let debug_messenger = if ENABLE_VALIDATION_LAYERS {
            Some(DebugMessenger::new(&entry, &instance)?)
        } else {
            None
        };

        let device = Device::new(&instance)?;

        Ok(Self {
            _entry: entry,
            instance,
            device,
            debug_messenger,
        })
    }

    pub fn create_instance(entry: &Entry) -> Result<Instance, Box<dyn Error>> {
        let app_name = CString::new("Playground").expect("Failed to build app_name CString");

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .api_version(vk::API_VERSION_1_3);

        // Currently hard coding required extensions for Wayland
        // TODO: Get required extensions from winit once setup with window ref
        let mut extension_names = vec![
            khr::surface::NAME.as_ptr(),
            khr::wayland_surface::NAME.as_ptr(),
        ];
        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(debug_utils::NAME.as_ptr());
        }

        // Get names of wanted validation layers
        // must be done out here to appease the lifetime gods
        let (_layer_names, layer_name_ptrs) = Self::get_layer_names_and_pointers();

        // Enable the required extensions in the instance
        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);
        if ENABLE_VALIDATION_LAYERS {
            Self::check_validation_layer_support(entry)?;
            instance_create_info = instance_create_info.enabled_layer_names(&layer_name_ptrs);
        }

        unsafe { Ok(entry.create_instance(&instance_create_info, None).unwrap()) }
    }

    pub(super) fn get_layer_names_and_pointers() -> (Vec<CString>, Vec<*const i8>) {
        let layer_names = REQUIRED_VALIDATION_LAYERS
            .iter()
            .map(|n| CString::new(*n).expect("Failed to build CString from layer name"))
            .collect::<Vec<_>>();
        let layer_name_ptrs = layer_names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();
        (layer_names, layer_name_ptrs)
    }

    fn check_validation_layer_support(entry: &Entry) -> Result<(), Box<dyn Error>> {
        // Check that the system supports all validation layers listed above [REQUIRED_VALIDATION_LAYERS]
        for layer in REQUIRED_VALIDATION_LAYERS.iter() {
            let found = unsafe {
                entry
                    .enumerate_instance_layer_properties()
                    .unwrap()
                    .iter()
                    .any(|l| {
                        let name = CStr::from_ptr(l.layer_name.as_ptr());
                        let name = name.to_str().expect("Failed to get layer name");
                        name == *layer
                    })
            };

            if !found {
                return Err(format!("Validation layer not supported: {layer}").into());
            }
        }

        Ok(())
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
