use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;



struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

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
        .find(|f| f.is_srgb())
        .unwrap_or(capabilities.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width: canvas.width(),
        height: canvas.height(),
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: capabilities.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let state = State {
        device,
        queue,
        surface,
        config,
    };

    render(&state)?;

    Ok(())
}

fn render(state: &State) -> Result<(), JsValue> {

    let output = match state.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(texture) => texture,

        wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,

        wgpu::CurrentSurfaceTexture::Timeout => {
            return Err(JsValue::from_str("Surface texture acquisition timed out"));
        }

        wgpu::CurrentSurfaceTexture::Occluded => {
            return Err(JsValue::from_str("Surface is occluded"));
        }

        wgpu::CurrentSurfaceTexture::Outdated => {
            return Err(JsValue::from_str("Surface is outdated"));
        }

        wgpu::CurrentSurfaceTexture::Lost => {
            return Err(JsValue::from_str("Surface was lost"));
        }

        wgpu::CurrentSurfaceTexture::Validation => {
            return Err(JsValue::from_str("Surface texture acquisition failed validation"));
        }
    };

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.05,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    state.queue.submit(Some(encoder.finish()));
    state.queue.present(output);

    Ok(())
}