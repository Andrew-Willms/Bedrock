use crate::buffers::Buffers;
use crate::simulation_parameters::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages};



pub(crate) struct ComputeState {
	pub(crate) pipeline: ComputePipeline,
	bind_group_a_to_b: BindGroup,
	bind_group_b_to_a: BindGroup,
	buffer_a_is_source: bool,
}



impl ComputeState {

	pub(crate) fn current_particle_buffer<'a>(&self, buffers: &'a Buffers) -> &'a Buffer {
		return if self.buffer_a_is_source {
			&buffers.particles_a
		} else {
			&buffers.particles_b
		}
	}

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

	pub(crate) fn new<'a>(device: &Device, buffers: &Buffers)-> ComputeState {

		let bind_group_layout = initialize_bind_group_layout(&device);
		let pipeline = initialize_pipeline(device, &bind_group_layout);

		return ComputeState {
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
						},
						BindGroupEntry {
							binding: 2,
							resource: buffers.parameters.as_entire_binding(),
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
						},
						BindGroupEntry {
							binding: 2,
							resource: buffers.parameters.as_entire_binding(),
						},
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
			label: Some("Compute Bind Group Layout"),
			entries: &[
				BindGroupLayoutEntry {
					binding: 0,
					visibility: ShaderStages::COMPUTE,
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
					visibility: ShaderStages::COMPUTE,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Storage {
							read_only: false,
						},
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
				BindGroupLayoutEntry {
					binding: 2,
					visibility: ShaderStages::COMPUTE,
					ty: BindingType::Buffer {
						ty: BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		},
	);
}

fn initialize_pipeline(device: &Device, bind_group_layout: &BindGroupLayout) -> ComputePipeline {

	let shader_module = device.create_shader_module(
		ShaderModuleDescriptor {
			label: Some("Particle Compute Shader"),
			source: ShaderSource::Wgsl(
				include_str!("shaders/compute_shader.wgsl").into()
			)
		}
	);

	let pipeline_layout = device.create_pipeline_layout(
		&PipelineLayoutDescriptor {
			label: Some("Compute Pipeline Layout"),
			bind_group_layouts: &[Some(&bind_group_layout)],
			immediate_size: 0,
		},
	);

	return device.create_compute_pipeline(
		&ComputePipelineDescriptor {
			label: Some("Particle Compute Pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader_module,
			entry_point: Some("main"),
			compilation_options: PipelineCompilationOptions {
				constants: &[
					("SIMULATION_WIDTH", SIMULATION_WIDTH as f64),
					("SIMULATION_HEIGHT", SIMULATION_HEIGHT as f64)
				],
				..Default::default()
			},
			cache: None,
		},
	);
}