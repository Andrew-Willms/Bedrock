use wasm_bindgen::JsValue;
use web_sys::{console, HtmlCanvasElement, Window};
use crate::gpu_state::GpuState;
use crate::simulation_parameters::SimulationParameters;
use crate::web_state::WebState;



pub(crate) struct State {
	
	pub(crate) web_state: WebState,
	pub(crate) gpu_state: GpuState,
	
	pub(crate) compute_pipeline: wgpu::ComputePipeline,
	pub(crate) compute_bind_group: wgpu::BindGroup,
	
	pub(crate) render_pipeline: wgpu::RenderPipeline,
	
	pub(crate) particle_count: u32,
	pub(crate) simulation_parameter_buffer: wgpu::Buffer,
	pub(crate) particle_buffer: wgpu::Buffer,
}



impl State {
	
	fn compute(&mut self, delta_time: f32) {
		
		let parameters = SimulationParameters {
			delta_time,
			_padding: [0.0; 3],
		};
		
		self.gpu_state.queue.write_buffer(
			&self.simulation_parameter_buffer,
			0,
			bytemuck::bytes_of(&parameters),
		);
		
		let mut encoder = self.gpu_state.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {
				label: Some("Compute Encoder"),
			},
		);
		
		{
			let mut compute_pass = encoder.begin_compute_pass(
				&wgpu::ComputePassDescriptor {
					label: Some("Particle Compute Pass"),
					timestamp_writes: None,
				},
			);
			
			compute_pass.set_pipeline(&self.compute_pipeline);
			compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
			
			let workgroup_count = self.particle_count.div_ceil(64);
			compute_pass.dispatch_workgroups(workgroup_count,1,1);
		}
		
		self.gpu_state.queue.submit(Some(encoder.finish()));
	}
	
	pub(crate) fn render(&mut self) -> Result<(), JsValue> {
		
		self.resize_if_necessary();
		
		let output = match self.gpu_state.surface.get_current_texture() {
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
				return Err(JsValue::from_str(
					"Surface texture acquisition failed validation",
				));
			}
		};
		
		let view = output
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		
		
		
		let parameters = SimulationParameters {
			delta_time: 1.0 / 60.0,
			_padding: [0.0; 3],
		};
		
		self.gpu_state.queue.write_buffer(
			&self.simulation_parameter_buffer,
			0,
			bytemuck::bytes_of(&parameters),
		);
		
		
		
		let mut encoder = self.gpu_state.device.create_command_encoder(
			&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			}
		);
		
		
		
		{
			let mut compute_pass = encoder.begin_compute_pass(
				&wgpu::ComputePassDescriptor {
					label: Some("Particle Compute Pass"),
					timestamp_writes: None,
				},
			);
			
			compute_pass.set_pipeline(&self.compute_pipeline);
			compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
			
			let workgroup_count = self.particle_count.div_ceil(64);
			compute_pass.dispatch_workgroups(workgroup_count,1,1);
		}
		
		
		
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					depth_slice: None,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.25,
							g: 0.25,
							b: 0.25,
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
			
			render_pass.set_pipeline(&self.render_pipeline);
			render_pass.set_vertex_buffer(0, self.particle_buffer.slice(..));
			render_pass.draw(0..self.particle_count, 0..1);
		}
		
		self.gpu_state.queue.submit(Some(encoder.finish()));
		self.gpu_state.queue.present(output);
		
		return Ok(());
	}
	
	pub(crate) fn resize_if_necessary(&mut self) {
		
		let bounding_rectangle = self.web_state.canvas.get_bounding_client_rect();
		let device_pixel_ratio = self.web_state.window.device_pixel_ratio();
		
		if (device_pixel_ratio == self.web_state.last_device_pixel_ratio) &&
			(bounding_rectangle.width() == self.web_state.last_canvas_css_width) &&
			(bounding_rectangle.height() == self.web_state.last_canvas_css_height) {
			
			return;
		}
		
		self.web_state.last_device_pixel_ratio = device_pixel_ratio;
		self.web_state.last_canvas_css_width = bounding_rectangle.width();
		self.web_state.last_canvas_css_height = bounding_rectangle.height();
		
		self.gpu_state.config.width = (bounding_rectangle.width() * device_pixel_ratio).round() as u32;
		self.gpu_state.config.height = (bounding_rectangle.height() * device_pixel_ratio).round() as u32;
		
		self.gpu_state.surface.configure(&self.gpu_state.device, &self.gpu_state.config);
		
		console::log_1(&"resize".into());
	}
	
}