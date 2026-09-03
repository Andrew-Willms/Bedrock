use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::console;
use crate::simulation_parameters::MIN_FRAME_DISPLAY_TIME;
use crate::state;



pub(crate) fn start_render_loop(state: state::State) {
	
	let state = Rc::new(RefCell::new(state));
	let state_clone = state.clone();
	
	let callback: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
	let callback_clone = callback.clone();
	
	let previous_timestamp: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.0));
	let previous_timestamp_clone = previous_timestamp.clone();
	
	*callback_clone.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
		
		let delta_time = {
			(timestamp - *previous_timestamp_clone.borrow()) / 1000.0
		};

		if delta_time > MIN_FRAME_DISPLAY_TIME {
			
			if let Err(error) = state_clone.borrow_mut().render() {
				console::error_1(&error);
			}
			
			*previous_timestamp_clone.borrow_mut() = timestamp;
			
		} //else { console::log_1(&"skipping frame".into()); }
		
		state_clone.borrow().web_state.window
			.request_animation_frame(
				callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()
			)
			.unwrap();
		
	}) as Box<dyn FnMut(f64)>));
	
	state.borrow().web_state.window
		.request_animation_frame(
			callback_clone.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
		)
		.unwrap();
}