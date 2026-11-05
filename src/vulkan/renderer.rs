use std::error::Error;
use std::ffi::CString;

use ash::khr;
use ash::{Entry, Instance, vk};

const ENABLE_VALIDATION_LAYERS: bool = true;

pub(crate) struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,
}

impl VulkanRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        log::info!("Creating Vulkan Application!");

        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry)?;

        Ok(Self {
            _entry: entry,
            instance,
        })
    }

    pub fn create_instance(entry: &Entry) -> Result<Instance, Box<dyn Error>> {
        let app_name = CString::new("Playground")?;

        let app_info = vk::ApplicationInfo::default()
            .application_name(app_name.as_c_str())
            .api_version(vk::API_VERSION_1_3);

        let extension_names = vec![
            khr::surface::NAME.as_ptr(),
            khr::wayland_surface::NAME.as_ptr(),
        ];

        let instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        Ok(unsafe { entry.create_instance(&instance_create_info, None) }?)
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
