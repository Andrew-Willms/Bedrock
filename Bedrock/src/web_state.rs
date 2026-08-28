use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, Window};



pub(crate) struct WebState {
	pub(crate) window: Window,
	pub(crate) canvas: HtmlCanvasElement,
	pub(crate) last_canvas_css_width: f64,
	pub(crate) last_canvas_css_height: f64,
	pub(crate) last_device_pixel_ratio: f64
}



pub(crate) fn get_web_state() -> Result<WebState, JsValue> {
	
	let window = web_sys::window().expect("Could not access the window.");
	let document = window.document().expect("Could not access the window's document.");
	let canvas = document
		.get_element_by_id("canvas")
		.expect("An element with id 'canvas' does not exist.")
		.dyn_into::<HtmlCanvasElement>()?;
	
	let canvas_bounding_rectangle = canvas.get_bounding_client_rect();
	let device_pixel_ratio = window.device_pixel_ratio();
	
	return Ok(WebState {
		window,
		canvas,
		last_device_pixel_ratio: device_pixel_ratio,
		last_canvas_css_width: canvas_bounding_rectangle.width(),
		last_canvas_css_height: canvas_bounding_rectangle.height(),
	});
}