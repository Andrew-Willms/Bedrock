
//pub(crate) const MAX_PARTICLE_COUNT: u32 = 65536;
pub(crate) const SIMULATION_WIDTH: f32 = 10.0;
pub(crate) const SIMULATION_HEIGHT: f32 = 10.0;


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SimulationParameters {
	pub(crate) delta_time: f32,
	pub(crate) _padding: [f32; 3],
}