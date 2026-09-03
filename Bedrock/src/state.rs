use wasm_bindgen::JsValue;
use web_sys::{console};
use wgpu::{Color, CommandEncoderDescriptor, ComputePassDescriptor, CurrentSurfaceTexture, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, StoreOp, TextureViewDescriptor};
use crate::buffers::Buffers;
use crate::compute_state::ComputeState;
use crate::gpu_state::GpuState;
use crate::neighbor_render_state::NeighborRenderState;
use crate::particle::NEIGHBOR_COUNT;
use crate::simulation_parameters::{SimulationParameters, SIMULATION_TIME_STEP};
use crate::web_state::WebState;



pub(crate) struct State {
	
	pub(crate) web_state: WebState,
	pub(crate) gpu_state: GpuState,

	pub(crate) particle_rendering_pipeline: RenderPipeline,

	pub(crate) buffers: Buffers,
	pub(crate) compute_state: ComputeState,
	pub(crate) neighbor_render_state: NeighborRenderState,
	
	pub(crate) particle_count: u32
}



impl State {
	
	pub(crate) fn render(&mut self) -> Result<(), JsValue> {
		
		self.resize_if_necessary();
		
		let output = match self.gpu_state.surface.get_current_texture() {
			CurrentSurfaceTexture::Success(texture) => texture,
			
			CurrentSurfaceTexture::Suboptimal(texture) => texture,
			
			CurrentSurfaceTexture::Timeout => {
				return Err(JsValue::from_str("Surface texture acquisition timed out"));
			}
			
			CurrentSurfaceTexture::Occluded => {
				return Err(JsValue::from_str("Surface is occluded"));
			}
			
			CurrentSurfaceTexture::Outdated => {
				return Err(JsValue::from_str("Surface is outdated"));
			}
			
			CurrentSurfaceTexture::Lost => {
				return Err(JsValue::from_str("Surface was lost"));
			}
			
			CurrentSurfaceTexture::Validation => {
				return Err(JsValue::from_str(
					"Surface texture acquisition failed validation",
				));
			}
		};
		
		let view = output.texture.create_view(&TextureViewDescriptor::default());
		
		let parameters = SimulationParameters {
			delta_time: SIMULATION_TIME_STEP,
			_padding: [0.0; 3],
		};
		
		self.gpu_state.queue.write_buffer(&self.buffers.parameters, 0, bytemuck::bytes_of(&parameters));
		
		let mut encoder = self.gpu_state.device.create_command_encoder(
			&CommandEncoderDescriptor {
				label: Some("Command Encoder"),
			}
		);
		
		// This scope is to ensure the mutable borrow of encoder (using when assigning to compute_pass)
		// is returned before it needs to be used for render_pass.
		{
			let mut compute_pass = encoder.begin_compute_pass(
				&ComputePassDescriptor {
					label: Some("Particle Compute Pass"),
					timestamp_writes: None,
				},
			);
			
			compute_pass.set_pipeline(&self.compute_state.pipeline);
			compute_pass.set_bind_group(0, self.compute_state.current_bind_group(), &[]);
			
			let workgroup_count = self.particle_count.div_ceil(64);
			compute_pass.dispatch_workgroups(workgroup_count,1,1);
			
			self.compute_state.swap_particle_buffer_and_bind_group();
		}
		
		// This scope is to ensure the mutable borrow of encoder (using when assigning to render_particles_pass)
		// is returned before the encoder is finished and submitted to the queue.
		{
			let mut render_particles_pass = encoder.begin_render_pass(
				&RenderPassDescriptor {
					label: Some("Render Particles"),
					color_attachments: &[Some(RenderPassColorAttachment {
						view: &view,
						depth_slice: None,
						resolve_target: None,
						ops: Operations {
							load: LoadOp::Clear(Color {
								r: 0.0,
								g: 0.0,
								b: 0.0,
								a: 1.0,
							}),
							store: StoreOp::Store,
						},
					})],
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
					multiview_mask: None,
				}
			);
			
			render_particles_pass.set_pipeline(&self.particle_rendering_pipeline);
			render_particles_pass.set_vertex_buffer(
				0, self.compute_state.current_particle_buffer(&self.buffers).slice(..));
			render_particles_pass.draw(0..self.particle_count, 0..1);
		}

		// This scope is to ensure the mutable borrow of encoder (using when assigning to render_pass)
		// is returned before the encoder is finished and submitted to the queue.
		{
			let mut render_neighbor_pass = encoder.begin_render_pass(
				&RenderPassDescriptor {
					label: Some("Render Neighbors"),
					color_attachments: &[Some(RenderPassColorAttachment {
						view: &view,
						depth_slice: None,
						resolve_target: None,
						ops: Operations {
							load: LoadOp::Load,
							store: StoreOp::Store,
						},
					})],
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
					multiview_mask: None,
				}
			);

			render_neighbor_pass.set_pipeline(&self.neighbor_render_state.pipeline);
			render_neighbor_pass.set_bind_group(0, self.neighbor_render_state.current_bind_group(), &[]);
			//render_neighbor_pass.draw(0..(self.particle_count * NEIGHBOR_COUNT as u32 * 2), 0..1);
			render_neighbor_pass.draw(0..(1 * NEIGHBOR_COUNT as u32 * 2), 0..1);

			self.neighbor_render_state.swap_particle_buffer_and_bind_group();
		}
		
		self.gpu_state.queue.submit(Some(encoder.finish()));
		self.gpu_state.queue.present(output);
		
		return Ok(());
	}
	
	fn resize_if_necessary(&mut self) {

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

		console::log_1(&"resizing".into());
	}
	
}