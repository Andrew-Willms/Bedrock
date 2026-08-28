mod state;
mod start_render_loop;
mod particle;
mod simulation_parameters;
mod web_state;
mod gpu_state;

use wasm_bindgen::prelude::*;
use wgpu::*;

use crate::gpu_state::get_gpu_state;
use crate::particle::{create_particle_buffer, Particle};
use crate::simulation_parameters::create_simulation_parameter_buffer;
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
    
    let particle_buffer = create_particle_buffer(&gpu_state.device, 1000);
    let simulation_parameter_buffer = create_simulation_parameter_buffer(&gpu_state.device);
    
    let (compute_pipeline, compute_bind_group) =
        initialize_compute(&gpu_state.device, &particle_buffer, &simulation_parameter_buffer);
    
    let render_pipeline = initialize_render_pipeline(&gpu_state.device, &gpu_state.config);
    
    let state = state::State {
        web_state,
        gpu_state,
        
        compute_pipeline,
        compute_bind_group,
        render_pipeline,
        
        particle_buffer,
        simulation_parameter_buffer,
        particle_count: 1000
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

fn initialize_compute(device: &Device, particle_buffer: &Buffer, simulation_parameter_buffer: &Buffer) -> (ComputePipeline, BindGroup) {
    
    let compute_shader = device.create_shader_module(
        ShaderModuleDescriptor {
            label: Some("Particle Compute Shader"),
            source: ShaderSource::Wgsl(
                include_str!("shaders/compute_shader.wgsl").into()
            )
        }
    );
    
    let compute_bind_group_layout = device.create_bind_group_layout(
        &BindGroupLayoutDescriptor {
            label: Some("Compute Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
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
                    binding: 1,
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
    
    let compute_pipeline_layout = device.create_pipeline_layout(
        &PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[Some(&compute_bind_group_layout)],
            immediate_size: 0,
        },
    );
    
    return (
        
        device.create_compute_pipeline(
            &ComputePipelineDescriptor {
                label: Some("Particle Compute Pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &compute_shader,
                entry_point: Some("main"),
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            },
        ),
        
        device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Compute Bind Group"),
                layout: &compute_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: particle_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: simulation_parameter_buffer.as_entire_binding(),
                    },
                ],
            },
        )
        
    );
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
                offset: 0,
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