use std::{
    error::Error,
    ffi::{CStr, c_void},
};

use ash::{Entry, Instance, ext::debug_utils, vk};

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const REQUIRED_VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];

unsafe extern "system" fn vulkan_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = unsafe { CStr::from_ptr((*callback_data).p_message).to_string_lossy() };

    // Log type based on message severity
    match severity {
        sev if sev.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) => {
            log::error!("[VULKAN: {:?}] {message}", typ)
        }
        sev if sev.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) => {
            log::warn!("[VULKAN: {:?}] {message}", typ)
        }
        sev if sev.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) => {
            log::info!("[VULKAN: {:?}] {message}", typ)
        }
        sev if sev.contains(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE) => {
            log::debug!("[VULKAN: {:?}] {message}", typ)
        }
        _ => {}
    }

    vk::FALSE
}

pub struct DebugMessenger {
    loader: debug_utils::Instance,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn new(entry: &Entry, instance: &Instance) -> Result<Self, Box<dyn Error>> {
        // Basically just say "give me all the debug messages"
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let loader = debug_utils::Instance::new(entry, instance);

        let messenger = unsafe { loader.create_debug_utils_messenger(&create_info, None)? };

        Ok(Self { loader, messenger })
    }

    pub fn destroy(&self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}

pub fn check_validation_layer_support(entry: &Entry) -> Result<(), Box<dyn Error>> {
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
