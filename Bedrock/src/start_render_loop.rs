use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::state;



pub(crate) fn start_render_loop(state: state::State) {
	
	let state = Rc::new(RefCell::new(state));
	let callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
	
	let state_clone = state.clone();
	let callback_clone = callback.clone();
	
	*callback_clone.borrow_mut() = Some(Closure::wrap(Box::new(move || {
		{
			if let Err(error) = state_clone.borrow_mut().render() {
				web_sys::console::error_1(&error);
			}
		}
		
		state_clone.borrow().window
			.request_animation_frame(
				callback.borrow().as_ref().unwrap().as_ref().unchecked_ref()
			)
			.unwrap();
		
	}) as Box<dyn FnMut()>));
	
	state.borrow().window
		.request_animation_frame(
			callback_clone.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
		)
		.unwrap();
}