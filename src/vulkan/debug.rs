use std::{
    error::Error,
    ffi::{CStr, c_void},
};

use ash::{Entry, Instance, ext::debug_utils, vk};

unsafe extern "system" fn vulkan_debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message = unsafe { CStr::from_ptr((*callback_data).p_message).to_string_lossy() };

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
