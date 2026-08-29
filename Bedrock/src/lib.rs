mod state;
mod start_render_loop;
mod particle;
mod simulation_parameters;
mod web_state;
mod gpu_state;
mod compute_state;



use wasm_bindgen::prelude::*;
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, BufferAddress, BufferBindingType, ColorTargetState, ColorWrites, ComputePipeline, ComputePipelineDescriptor, Device, FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, SurfaceConfiguration, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode};
use crate::compute_state::initialize_compute_state;
use crate::gpu_state::get_gpu_state;
use crate::particle::{Particle};
use crate::simulation_parameters::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use crate::web_state::get_web_state;



// For debug build:
// wasm-pack build --target web --dev -- --features debug-logging
// python3 -m http.server 8000
// http://localhost:8000/

// For release build:
// wasm-pack build --target web
// python3 -m http.server 8000
// http://localhost:8000/



#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
    
    initialize_logging_in_debug()?;
    
    let web_state = get_web_state()?;
    let gpu_state = get_gpu_state(&web_state).await?;
    
    let layout = initialize_layout(&gpu_state.device)?;
    
    let compute_pipeline = initialize_compute_pipeline(&gpu_state.device, &layout)?;
    let render_pipeline = initialize_render_pipeline(&gpu_state.device, &gpu_state.config);
    
    let particle_count: usize = 1000;
    let compute_state = initialize_compute_state(&gpu_state.device, particle_count, &layout)?;
    
    let state = state::State {
        web_state,
        gpu_state,
        compute_pipeline,
        render_pipeline,
        compute_state,
        particle_count: particle_count as u32
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

fn initialize_layout(device: &Device) -> Result<BindGroupLayout, JsValue> {
    
    return Ok(device.create_bind_group_layout(
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
    ));
}

fn initialize_compute_pipeline(device: &Device, compute_bind_group_layout: &BindGroupLayout) -> Result<ComputePipeline, JsValue> {
    
    let compute_shader = device.create_shader_module(
        ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: ShaderSource::Wgsl(
                include_str!("shaders/compute_shader.wgsl").into()
            )
        }
    );
    
    let compute_pipeline_layout = device.create_pipeline_layout(
        &PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[Some(&compute_bind_group_layout)],
            immediate_size: 0,
        },
    );
    
    return Ok(device.create_compute_pipeline(
        &ComputePipelineDescriptor {
            label: Some("Particle Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
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
    ));
}

fn initialize_render_pipeline(device: &Device, config: &SurfaceConfiguration) -> RenderPipeline {
    
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("Particle Shader"),
        source: ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into())
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
            label: Some("Triangle Render Pipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("main_vertex_shader"),
                buffers: &[Some(particle_vertex_layout)],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("main_fragment_shader"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions {
                    constants: &[
                        ("SIMULATION_WIDTH", SIMULATION_WIDTH as f64),
                        ("SIMULATION_HEIGHT", SIMULATION_HEIGHT as f64)
                    ],
                    ..Default::default()
                },
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