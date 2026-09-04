//pub(crate) const MAX_PARTICLE_COUNT: u32 = u16::MAX as u32;
pub(crate) const SIMULATION_WIDTH: f32 = 10.0;
pub(crate) const SIMULATION_HEIGHT: f32 = 8.0;
pub(crate) const SIMULATION_TIME_STEP: f32 = 1.0 / 60.0;
pub(crate) const MIN_FRAME_DISPLAY_TIME: f64 = 1.0 / 12.0; // Some frames are skipped at 120 fps, non are at 240 fps.

pub(crate) const PARTICLE_COUNT: usize = 50;


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct SimulationParameters {
	pub(crate) delta_time: f32,
	pub(crate) _padding: [f32; 3],
}