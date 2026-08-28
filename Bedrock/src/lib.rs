mod state;
mod start_render_loop;
mod particle;

use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use crate::particle::{create_particle_buffer, Particle};



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

    #[cfg(feature = "debug-logging")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    }

    let window = web_sys::window().expect("Could not access the window.");
    let document = window.document().expect("Could not access the window's document.");

    let canvas = document
        .get_element_by_id("canvas")
        .expect("An element with id 'canvas' does not exist.")
        .dyn_into::<HtmlCanvasElement>()?;
    
    let canvas_bounding_rectangle = canvas.get_bounding_client_rect();
    let device_pixel_ratio = window.device_pixel_ratio();
    
    let width = (canvas_bounding_rectangle.width() * device_pixel_ratio).round() as u32;
    let height = (canvas_bounding_rectangle.height() * device_pixel_ratio).round() as u32;
    
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::BROWSER_WEBGPU,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });

    let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .expect("Could not create surface");

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await
        .expect("No suitable GPU adapters found.");

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .expect("Unable to create GPU device.");

    let capabilities = surface.get_capabilities(&adapter);

    let format = capabilities
        .formats
        .iter()
        .copied()
        .find(|f| return f.is_srgb())
        .unwrap_or(capabilities.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width,
        height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("shaders/shader.wgsl").into()
        ),
    });
    
    let particle_buffer = create_particle_buffer(&device, 1000);
    
    let particle_vertex_layout = wgpu::VertexBufferLayout {
        array_stride: size_of::<Particle>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
        ],
    };
    
    let render_pipeline = device.create_render_pipeline(
        &wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Render Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("main_vertex_shader"),
                buffers: &[Option::from(particle_vertex_layout)],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("main_fragment_shader"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        },
    );

    let state = state::State {
        window,
        canvas,
        last_device_pixel_ratio: device_pixel_ratio,
        last_canvas_css_width: canvas_bounding_rectangle.width(),
        last_canvas_css_height: canvas_bounding_rectangle.height(),
        device,
        queue,
        surface,
        config,
        render_pipeline,
        particle_buffer,
        particle_count: 1000
    };

    start_render_loop::start_render_loop(state);

    return Ok(());
}