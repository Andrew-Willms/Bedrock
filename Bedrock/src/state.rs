use wasm_bindgen::JsValue;



pub(crate) struct State {
	pub(crate) device: wgpu::Device,
	pub(crate) queue: wgpu::Queue,
	pub(crate) surface: wgpu::Surface<'static>,
	pub(crate) config: wgpu::SurfaceConfiguration,
	pub(crate) render_pipeline: wgpu::RenderPipeline,
}



impl State {
	
	pub(crate) fn render(&mut self) -> Result<(), JsValue> {
		let output = match self.surface.get_current_texture() {
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
		
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});
		
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					depth_slice: None,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.5,
							g: 0.5,
							b: 0.5,
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
			render_pass.draw(0..3, 0..1);
		}
		
		self.queue.submit(Some(encoder.finish()));
		self.queue.present(output);
		
		Ok(())
	}
	
}