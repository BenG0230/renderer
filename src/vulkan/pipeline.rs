use std::{
    error::Error,
    ffi::{CStr, CString},
    io::Cursor,
};

use ash::vk;

use crate::vulkan::{device::Device, renderer::Vertex, swapchain::Swapchain};

pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    shader_module: vk::ShaderModule,
}

impl Pipeline {
    pub fn new(swapchain: &Swapchain, device: &Device) -> Result<Self, Box<dyn Error>> {
        let mut shader_byte = std::fs::File::open(
            "/home/jam/Documents/Programming/Rust/renderer/renderer/examples/hello_tri/shader.spv",
        )?;
        let shader_source = ash::util::read_spv(&mut shader_byte)?;

        let shader_create_info = vk::ShaderModuleCreateInfo::default().code(&shader_source);
        let shader_module = unsafe {
            device
                .device
                .create_shader_module(&shader_create_info, None)
        }?;

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();

        let layout = unsafe {
            device
                .device
                .create_pipeline_layout(&pipeline_layout_info, None)?
        };

        let vertex_binding = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let descriptions = [
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(8),
        ];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding))
            .vertex_attribute_descriptions(&descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let module_name = CString::new("main")?;

        let shader_stages = vec![
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(shader_module)
                .name(module_name.as_c_str()),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(shader_module)
                .name(module_name.as_c_str()),
        ];

        let viewport_info = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let dynamic_states = vec![vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let mut rendering_create_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(std::slice::from_ref(&swapchain.image_format));

        let blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA);
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&blend_attachment));

        let rasterisation_state =
            vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterisation_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .push_next(&mut rendering_create_info);

        let pipeline = unsafe {
            device
                .device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&create_info),
                    None,
                )
                .unwrap()[0]
        };

        Ok(Self {
            pipeline,
            layout,
            shader_module,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.device.destroy_pipeline(self.pipeline, None);
            device.device.destroy_pipeline_layout(self.layout, None);
            device
                .device
                .destroy_shader_module(self.shader_module, None);
        };
    }
}
