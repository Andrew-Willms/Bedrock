use crate::buffers::Buffers;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, SurfaceConfiguration, VertexState};
use crate::simulation_parameters::{SIMULATION_HEIGHT, SIMULATION_WIDTH};



pub(crate) struct NeighborRenderState {
	pub(crate) pipeline: RenderPipeline,
	bind_group_a_to_b: BindGroup,
	bind_group_b_to_a: BindGroup,
	buffer_a_is_source: bool,
}



impl NeighborRenderState {
	
	pub(crate) fn current_bind_group(&self) -> &BindGroup {
		return if self.buffer_a_is_source {
			&self.bind_group_a_to_b
		} else {
			&self.bind_group_b_to_a
		}
	}
	
	pub(crate) fn swap_particle_buffer_and_bind_group(&mut self) {
		self.buffer_a_is_source = !self.buffer_a_is_source;
	}
	
	pub(crate) fn new<'a>(device: &Device, config: &SurfaceConfiguration, buffers: &'a Buffers) -> NeighborRenderState {
		
		let bind_group_layout = initialize_bind_group_layout(&device);
		let pipeline = initialize_pipeline(device, &config, &bind_group_layout);
		
		return NeighborRenderState {
			pipeline,
			bind_group_a_to_b: device.create_bind_group(
				&BindGroupDescriptor {
					label: Some("Particles A to B"),
					layout: &bind_group_layout,
					entries: &[
						BindGroupEntry {
							binding: 0,
							resource: buffers.particles_a.as_entire_binding(),
						},
						BindGroupEntry {
							binding: 1,
							resource: buffers.particles_b.as_entire_binding(),
						}
					],
				},
			),
			bind_group_b_to_a: device.create_bind_group(
				&BindGroupDescriptor {
					label: Some("Particles B to A"),
					layout: &bind_group_layout,
					entries: &[
						BindGroupEntry {
							binding: 0,
							resource: buffers.particles_b.as_entire_binding(),
						},
						BindGroupEntry {
							binding: 1,
							resource: buffers.particles_a.as_entire_binding(),
						}
					],
				},
			),
			buffer_a_is_source: true
		};
	}
	
}



fn initialize_bind_group_layout(device: &Device) -> BindGroupLayout {
	
	return device.create_bind_group_layout(
		&BindGroupLayoutDescriptor {
			label: Some("Neighbor Render Bind Group Layout"),
			entries: &[
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::VERTEX,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Storage {
							read_only: true,
						},
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 1,
					visibility: ShaderStages::VERTEX,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Storage {
							read_only: true,
						},
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				}
			],
		}
	);
}

fn initialize_pipeline(device: &Device, config: &SurfaceConfiguration, bind_group_layout: &BindGroupLayout) -> RenderPipeline {
	
	let shader_module = device.create_shader_module(ShaderModuleDescriptor {
		label: Some("Neighbor Shader"),
		source: ShaderSource::Wgsl(include_str!("shaders/neighbor_shader.wgsl").into())
	});
	
	let neighbor_render_pipeline_layout = device.create_pipeline_layout(
		&PipelineLayoutDescriptor {
			label: Some("Neighbor Render Pipeline Layout"),
			bind_group_layouts: &[Some(bind_group_layout)],
			immediate_size: 0,
		},
	);
	
	return device.create_render_pipeline(
		&RenderPipelineDescriptor {
			label: Some("Neighbor Render Pipeline"),
			layout: Some(&neighbor_render_pipeline_layout),
			vertex: VertexState {
				module: &shader_module,
				entry_point: Some("main_neighbor_vertex_shader"),
				buffers: &[],
				compilation_options: PipelineCompilationOptions {
					constants: &[
						("SIMULATION_WIDTH", SIMULATION_WIDTH as f64),
						("SIMULATION_HEIGHT", SIMULATION_HEIGHT as f64)
					],
					..Default::default()
				}
			},
			fragment: Some(FragmentState {
				module: &shader_module,
				entry_point: Some("main_neighbor_fragment_shader"),
				targets: &[Some(ColorTargetState {
					format: config.format,
					blend: Some(BlendState::ALPHA_BLENDING),
					write_mask: ColorWrites::ALL,
				})],
				compilation_options: PipelineCompilationOptions::default(),
			}),
			primitive: PrimitiveState {
				topology: PrimitiveTopology::LineList,
				..Default::default()
			},
			depth_stencil: None,
			multisample: MultisampleState::default(),
			multiview_mask: None,
			cache: None,
		},
	);
}