use wgpu::{Buffer, Device};
use wgpu::util::DeviceExt;



#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SimulationParameters {
	pub(crate) delta_time: f32,
	pub(crate) _padding: [f32; 3],
}



pub(crate) fn create_simulation_parameter_buffer(device: &Device) -> Buffer {
	
	let simulation_params = SimulationParameters {
		delta_time: 0.0,
		_padding: [0.0; 3],
	};
	
	return device.create_buffer_init(
		&wgpu::util::BufferInitDescriptor {
			label: Some("Simulation Parameters"),
			contents: bytemuck::bytes_of(&simulation_params),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
		},
	);
}