use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state;



pub(crate) fn start_render_loop(state: state::State) {
	
	let state = Rc::new(RefCell::new(state));
	let window = web_sys::window().unwrap();
	let callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
	
	let callback_clone = callback.clone();
	let state_clone = state.clone();
	let window_clone = window.clone();
	
	*callback_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
		{
			let mut state = state_clone.borrow_mut();
			
			if let Err(error) = state.render() {
				web_sys::console::error_1(&error);
			}
		}
		
		window_clone
			.request_animation_frame(
				callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()
			)
			.unwrap();
		
	}) as Box<dyn FnMut()>));
	
	window
		.request_animation_frame(
			callback_clone
				.borrow()
				.as_ref()
				.unwrap()
				.as_ref()
				.unchecked_ref(),
		)
		.unwrap();
	
}