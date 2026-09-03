mod state;
mod start_render_loop;
mod particle;
mod simulation_parameters;
mod web_state;
mod gpu_state;
mod compute_state;
mod neighbor_render_state;
mod buffers;

use crate::buffers::{Buffers};
use crate::compute_state::{ComputeState};
use crate::gpu_state::get_gpu_state;
use crate::neighbor_render_state::{NeighborRenderState};
use crate::particle::Particle;
use crate::simulation_parameters::{PARTICLE_COUNT, SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::web_state::get_web_state;
use wasm_bindgen::prelude::*;
use wgpu::{BlendState, BufferAddress, ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineCompilationOptions, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, SurfaceConfiguration, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};



// For debug build:     wasm-pack build --target web --dev -- --features debug-logging
// For release build:   wasm-pack build --target web
// python3 -m http.server 8000
// http://localhost:8000/



#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
	
	initialize_logging_in_debug()?;
	
	let web_state = get_web_state()?;
	let gpu_state = get_gpu_state(&web_state).await?;
	
	let buffers = Buffers::new(&gpu_state.device, PARTICLE_COUNT);
	
	let compute_state = ComputeState::new(&gpu_state.device, &buffers);
	let neighbor_render_state = NeighborRenderState::new(&gpu_state.device, &gpu_state.config, &buffers);
	let particle_rendering_pipeline = initialize_particle_render_pipeline(&gpu_state.device, &gpu_state.config);
	
	let state = state::State {
		web_state,
		gpu_state,
		particle_rendering_pipeline,
		buffers,
		compute_state,
		neighbor_render_state,
		particle_count: PARTICLE_COUNT as u32
	};
	
	start_render_loop::start_render_loop(state);
	
	return Ok(());
}



fn initialize_logging_in_debug() -> Result<(), JsValue> {
	
	#[cfg(feature = "debug-logging")] {
		std::panic::set_hook(Box::new(console_error_panic_hook::hook));
		console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
	}
	
	return Ok(());
}

fn initialize_particle_render_pipeline(device: &Device, config: &SurfaceConfiguration) -> RenderPipeline {
	
	let shader_module = device.create_shader_module(ShaderModuleDescriptor {
		label: Some("Particle Shader"),
		source: ShaderSource::Wgsl(include_str!("shaders/particle_shader.wgsl").into())
	});
	
	let particle_vertex_layout = VertexBufferLayout {
		array_stride: size_of::<Particle>() as BufferAddress,
		step_mode: VertexStepMode::Vertex,
		attributes: &[
			VertexAttribute {
				format: VertexFormat::Float32x2,
				offset: 8,
				shader_location: 0,
			},
		],
	};
	
	return device.create_render_pipeline(
		&RenderPipelineDescriptor {
			label: Some("Particle Render Pipeline"),
			layout: None,
			vertex: VertexState {
				module: &shader_module,
				entry_point: Some("main_particle_vertex_shader"),
				buffers: &[Some(particle_vertex_layout)],
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
				entry_point: Some("main_particle_fragment_shader"),
				targets: &[Some(ColorTargetState {
					format: config.format,
					blend: Some(BlendState::REPLACE),
					write_mask: ColorWrites::ALL,
				})],
				compilation_options: PipelineCompilationOptions::default(),
			}),
			primitive: PrimitiveState {
				topology: PrimitiveTopology::PointList,
				..Default::default()
			},
			depth_stencil: None,
			multisample: MultisampleState::default(),
			multiview_mask: None,
			cache: None,
		},
	);
}

