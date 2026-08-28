use wasm_bindgen::JsValue;
use wgpu::{Adapter, Backends, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, PresentMode, RequestAdapterOptions, Surface, SurfaceColorSpace, SurfaceConfiguration, SurfaceTarget, TextureUsages};
use crate::web_state::WebState;



pub(crate) struct GpuState {
	pub(crate) device: wgpu::Device,
	pub(crate) queue: wgpu::Queue,
	pub(crate) surface: Surface<'static>,
	pub(crate) config: SurfaceConfiguration,
}



pub(crate) async fn get_gpu_state(web_state: &WebState) -> Result<GpuState, JsValue>{
	
	let instance = Instance::new(InstanceDescriptor {
		backends: Backends::BROWSER_WEBGPU,
		..InstanceDescriptor::new_without_display_handle()
	});
	
	let surface = instance
		.create_surface(SurfaceTarget::Canvas(web_state.canvas.clone()))
		.expect("Could not create surface");
	
	let adapter = instance
		.request_adapter(&RequestAdapterOptions {
			power_preference: PowerPreference::HighPerformance,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
			apply_limit_buckets: false,
		})
		.await
		.expect("No suitable GPU adapters found.");
	
	let (device, queue) = adapter
		.request_device(&DeviceDescriptor::default())
		.await
		.expect("Unable to create GPU device.");
	
	let config = initialize_config(&surface, &adapter, web_state);
	surface.configure(&device, &config);
	
	return Ok(GpuState {
		device,
		queue,
		surface,
		config,
	});
}



fn initialize_config(surface: &Surface, adapter: &Adapter, web_state: &WebState) -> SurfaceConfiguration {
	
	let width = (web_state.last_canvas_css_width * web_state.last_device_pixel_ratio).round() as u32;
	let height = (web_state.last_canvas_css_height * web_state.last_device_pixel_ratio).round() as u32;
	
	let capabilities = surface.get_capabilities(&adapter);
	
	let format = capabilities
		.formats
		.iter()
		.copied()
		.find(|f| return f.is_srgb())
		.unwrap_or(capabilities.formats[0]);
	
	return SurfaceConfiguration {
		usage: TextureUsages::RENDER_ATTACHMENT,
		format,
		color_space: SurfaceColorSpace::Auto,
		width,
		height,
		present_mode: PresentMode::AutoVsync,
		alpha_mode: capabilities.alpha_modes[0],
		view_formats: vec![],
		desired_maximum_frame_latency: 2,
	};
}