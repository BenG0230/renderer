use std::mem::ManuallyDrop;
use std::time::{Duration, Instant};
use std::{error::Error, ffi::CString};

use ash::{Entry, Instance, ext::debug_utils, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle};

use crate::vulkan::allocation::{Buffer, VulkanAllocator};
use crate::vulkan::commands::Command;
use crate::vulkan::frame::Frame;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::surface::Surface;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::{debug::DebugMessenger, device::Device};

const FRAMES_IN_FLIGHT: u32 = 2;
// const VERTICES: [Vertex; 3] = [
//     Vertex {
//         pos: [-1.0, -1.0],
//         color: [1.0, 0.0, 0.0],
//     },
//     Vertex {
//         pos: [1.0, -1.0],
//         color: [0.0, 1.0, 0.0],
//     },
//     Vertex {
//         pos: [0.0, 1.0],
//         color: [0.0, 0.0, 1.0],
//     },
// ];

pub(crate) struct VulkanRenderer {
    _entry: Entry,
    instance: Instance,

    device: Device,

    swapchain: Swapchain,

    surface: Surface,
    debug_messenger: Option<DebugMessenger>,

    command_pool: Command,

    frames: Vec<Frame>,
    image_fences: Vec<vk::Fence>,
    current_frame: usize,

    pipeline: Pipeline,

    allocator: ManuallyDrop<VulkanAllocator>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: usize,
    pub resize_requested: bool,
    pub resize_request_time: Instant,
    pub width: u32,
    pub height: u32,

    time: Instant,
}

impl VulkanRenderer {
    pub fn new(
        window: &(impl HasDisplayHandle + HasWindowHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Creating Vulkan Application!");

        let display_handle = window.display_handle();
        if let Err(e) = display_handle {
            return Err(format!("Error getting window display handle: {}", e).into());
        }

        let entry = unsafe { ash::Entry::load() }?;
        let instance = Self::create_instance(&entry, display_handle.unwrap().as_raw())?;
        let debug_messenger = if super::debug::ENABLE_VALIDATION_LAYERS {
            Some(DebugMessenger::new(&entry, &instance)?)
        } else {
            None
        };

        let surface = Surface::new(&entry, &instance, window)?;
        let device = Device::new(&instance, &surface)?;

        let allocator = ManuallyDrop::new(VulkanAllocator::new(&instance, &device)?);

        let swapchain = Swapchain::new(
            &instance,
            &device,
            &surface,
            &allocator,
            width,
            height,
            vk::SwapchainKHR::null(),
        )?;

        let command_pool = Command::new(&device)?;
        let cmd_buffers = command_pool.allocate_buffer(&device, FRAMES_IN_FLIGHT)?;

        let mut frames = Vec::new();
        for i in 0..FRAMES_IN_FLIGHT {
            log::debug!("Creating frame {i}");
            frames.push(Frame::new(cmd_buffers[i as usize], &device)?);
        }

        let image_fences = vec![vk::Fence::null(); swapchain.images.len()];

        let pipeline = Pipeline::new(&swapchain, &device)?;

        let (vertices, indices) = Self::load_model();

        let vertex_buffer = Buffer::vertex_buffer(&allocator, &vertices)?;
        let index_buffer = Buffer::index_buffer(&allocator, &indices)?;

        Ok(Self {
            _entry: entry,
            instance,
            device,
            swapchain,
            surface,
            debug_messenger,
            command_pool,
            frames,
            image_fences,
            current_frame: 0,
            pipeline,
            allocator,
            vertex_buffer,
            index_buffer,
            index_count: indices.len(),
            resize_requested: false,
            resize_request_time: Instant::now(),
            width,
            height,
            time: Instant::now(),
        })
    }
    fn load_model() -> (Vec<Vertex>, Vec<u32>) {
        let (models, materials) = tobj::load_obj(
            "./examples/hello_tri/Margeship.obj",
            &tobj::GPU_LOAD_OPTIONS,
        )
        .unwrap();

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut model_index_start = 0;

        for (i, m) in models.iter().enumerate() {
            let mesh = &m.mesh;
            log::info!(
                "Mesh data = {}-{}-{}",
                mesh.positions.len(),
                mesh.normals.len(),
                mesh.texcoords.len()
            );
            if mesh.positions.len() != mesh.normals.len() {
                log::error!(
                    "Error reading mesh '{}', Positions and normals have different lengths",
                    m.name
                );
                continue;
            }

            for vi in 0..mesh.positions.len() / 3 {
                vertices.push(Vertex {
                    pos: glam::Vec3::new(
                        mesh.positions[vi * 3],
                        mesh.positions[vi * 3 + 1],
                        mesh.positions[vi * 3 + 2],
                    ),
                    normal: glam::Vec3::new(
                        mesh.normals[vi * 3],
                        mesh.normals[vi * 3 + 1],
                        mesh.normals[vi * 3 + 2],
                    ),
                });
            }

            for index in &mesh.indices {
                indices.push(*index + model_index_start);
            }

            model_index_start = vertices.len() as u32;
        }

        (vertices, indices)
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

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::default()
        };

        // Get names of wanted validation layers
        // must be done out here to appease the lifetime gods
        // layer_names must not be discarded as well
        let (_layer_names, layer_name_ptrs) = Self::get_layer_names_and_pointers();

        // Enable the required extensions in the instance
        let mut instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

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

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }

        // 10ms debounce time
        if self.resize_requested && self.resize_request_time.elapsed() > Duration::from_millis(150)
        {
            // wait for device to be idle to create new swapchain and destroy old one
            unsafe { self.device.device.device_wait_idle() }?;

            let new_swapchain = Swapchain::new(
                &self.instance,
                &self.device,
                &self.surface,
                &self.allocator,
                self.width,
                self.height,
                self.swapchain.swapchain,
            )?;

            let mut old_swapchain = std::mem::replace(&mut self.swapchain, new_swapchain);
            old_swapchain.destroy(&self.allocator, &self.device);

            self.resize_requested = false;
        }

        let frame = &self.frames[self.current_frame];
        unsafe {
            self.device.device.wait_for_fences(
                std::slice::from_ref(&frame.in_flight_fence),
                true,
                u64::MAX,
            )?
        };

        let (image_index, _) = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.swapchain,
                u64::MAX,
                frame.image_available,
                vk::Fence::null(),
            )?
        };
        let image_fence = self.image_fences[image_index as usize];
        if image_fence != vk::Fence::null() {
            unsafe {
                self.device.device.wait_for_fences(
                    std::slice::from_ref(&image_fence),
                    true,
                    u64::MAX,
                )?
            };
        }
        self.image_fences[image_index as usize] = frame.in_flight_fence;

        let command_buffer = frame.command_buffer;
        unsafe {
            self.device
                .device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())?
        };

        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe {
            self.device
                .device
                .begin_command_buffer(command_buffer, &begin_info)?
        };

        let image_barriers = [
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::NONE)
                .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags2::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                )
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .image(self.swapchain.images[image_index as usize])
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                ),
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS)
                .dst_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .image(self.swapchain.depth_images[image_index as usize].image)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
                        .level_count(1)
                        .layer_count(1),
                ),
        ];

        let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&image_barriers);

        unsafe {
            self.device
                .device
                .cmd_pipeline_barrier2(command_buffer, &dependency_info)
        };

        let colour_attachment_info = vk::RenderingAttachmentInfo::default()
            .image_view(self.swapchain.image_views[image_index as usize])
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            });
        let depth_attachment_info = vk::RenderingAttachmentInfo::default()
            .image_view(self.swapchain.depth_image_views[image_index as usize])
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let rendering_info = vk::RenderingInfo::default()
            .render_area(self.swapchain.extent.into())
            .layer_count(1)
            .color_attachments(std::slice::from_ref(&colour_attachment_info))
            .depth_attachment(&depth_attachment_info);

        unsafe {
            self.device
                .device
                .cmd_begin_rendering(command_buffer, &rendering_info)
        };

        let vp = vk::Viewport::default()
            .x(0.0)
            .y(self.swapchain.extent.height as f32)
            .width(self.swapchain.extent.width as f32)
            .height(-(self.swapchain.extent.height as f32))
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor: vk::Rect2D = self.swapchain.extent.into();
        unsafe {
            self.device
                .device
                .cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&vp));
            self.device
                .device
                .cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));
        }

        unsafe {
            self.device.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            self.device.device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                std::slice::from_ref(&self.vertex_buffer.buffer),
                &[0],
            );
            self.device.device.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );

            let constants = PushConstants {
                resolution: glam::Vec2::new(self.width as f32, self.height as f32),
                time: self.time.elapsed().as_millis() as f32,
            };
            if self.time.elapsed().as_secs() > 10 {
                self.time = Instant::now();
            }

            let test = std::slice::from_raw_parts(
                &constants as *const PushConstants as *const u8,
                size_of::<PushConstants>(),
            );

            self.device.device.cmd_push_constants(
                command_buffer,
                self.pipeline.layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                &*test,
            );

            self.device.device.cmd_draw_indexed(
                command_buffer,
                self.index_count as u32,
                1,
                0,
                0,
                0,
            );
        }

        unsafe {
            self.device.device.cmd_end_rendering(command_buffer);
        }

        let image_barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .image(self.swapchain.images[image_index as usize])
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            );

        let dependency_info = vk::DependencyInfo::default()
            .image_memory_barriers(std::slice::from_ref(&image_barrier));

        unsafe {
            self.device
                .device
                .cmd_pipeline_barrier2(command_buffer, &dependency_info);
        };

        unsafe { self.device.device.end_command_buffer(command_buffer)? }

        let wait_semaphore_info = vk::SemaphoreSubmitInfo::default()
            .semaphore(frame.image_available)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

        let command_buffer_submit_info =
            vk::CommandBufferSubmitInfo::default().command_buffer(command_buffer);

        let signal_semaphore_info = vk::SemaphoreSubmitInfo::default()
            .semaphore(self.swapchain.render_finished[image_index as usize])
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

        let submit_info = vk::SubmitInfo2::default()
            .wait_semaphore_infos(std::slice::from_ref(&wait_semaphore_info))
            .command_buffer_infos(std::slice::from_ref(&command_buffer_submit_info))
            .signal_semaphore_infos(std::slice::from_ref(&signal_semaphore_info));

        unsafe {
            self.device
                .device
                .reset_fences(std::slice::from_ref(&frame.in_flight_fence))?;

            self.device.device.queue_submit2(
                self.device.graphics_queue,
                std::slice::from_ref(&submit_info),
                frame.in_flight_fence,
            )?
        };

        self.current_frame = (self.current_frame + 1) % self.frames.len();

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(std::slice::from_ref(
                &self.swapchain.render_finished[image_index as usize],
            ))
            .swapchains(std::slice::from_ref(&self.swapchain.swapchain))
            .image_indices(std::slice::from_ref(&image_index));

        unsafe {
            self.swapchain
                .loader
                .queue_present(self.device.present_queue, &present_info)?
        };

        Ok(())
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

            self.vertex_buffer.destroy(&self.allocator);
            self.index_buffer.destroy(&self.allocator);

            log::debug!("Dropping pipeline");
            self.pipeline.destroy(&self.device);

            log::debug!("Dropping frames");
            for frame in &self.frames {
                frame.destroy(&self.device);
            }

            log::debug!("Dropping command pool");
            self.command_pool.destroy(&self.device);

            self.swapchain.destroy(&self.allocator, &self.device);

            ManuallyDrop::drop(&mut self.allocator);

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

#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
    pub normal: glam::Vec3,
}

#[repr(C)]
pub struct PushConstants {
    pub resolution: glam::Vec2,
    pub time: f32,
}
