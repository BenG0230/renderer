use std::error::Error;
use std::ffi::{CStr, CString};

use ash::ext::debug_report;
use ash::khr;
use ash::{Entry, Instance, vk};

const ENABLE_VALIDATION_LAYERS: bool = true;
const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

pub(crate) struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
}

impl VulkanRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        log::info!("Creating Vulkan Application!");

        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry);

        Ok(Self {
            _entry: entry,
            instance,
        })
    }

    pub fn create_instance(entry: &Entry) -> Instance {
        let app_name = CString::new("Playground").expect("Failed to build app_name CString");

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .api_version(vk::API_VERSION_1_3);

        let mut extension_names = vec![
            khr::surface::NAME.as_ptr(),
            khr::wayland_surface::NAME.as_ptr(),
        ];
        if ENABLE_VALIDATION_LAYERS {
            extension_names.push(debug_report::NAME.as_ptr());
        }

        let layer_names = REQUIRED_VALIDATION_LAYERS
            .iter()
            .map(|l| CString::new(*l).expect("Failed to build CString from layer name"))
            .collect::<Vec<_>>();
        let layer_name_ptrs = layer_names.iter().map(|n| n.as_ptr()).collect::<Vec<_>>();

        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);
        if ENABLE_VALIDATION_LAYERS {
            Self::check_validation_layer_support(entry);
            instance_create_info = instance_create_info.enabled_layer_names(&layer_name_ptrs);
        }

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn check_validation_layer_support(entry: &Entry) -> bool {
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
                panic!("Validation layer not supported: {}", layer);
            }
        }

        false
    }

    pub fn run(&self) {
        log::info!("Running Application!");
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        log::debug!("Dropping vulkan application");
        unsafe { self.instance.destroy_instance(None) };
    }
}
