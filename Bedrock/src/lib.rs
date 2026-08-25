use std::sync::Arc;
use wasm_bindgen::prelude::*;
// use winit::{
//     event::*,
//     event_loop::{ControlFlow, EventLoop},
//     window::WindowBuilder,
// };



// struct State {
//     surface: wgpu::Surface<'static>,
//     device: wgpu::Device,
//     queue: wgpu::Queue,
//     config: wgpu::SurfaceConfiguration,
//     size: winit::dpi::PhysicalSize<u32>,
//     render_pipeline: wgpu::RenderPipeline,
// }



#[wasm_bindgen(start)]
pub fn main() {
    // Get the browser's document.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document");

    // Create an <h1> element.
    let heading = document
        .create_element("h1")
        .expect("failed to create heading");

    heading.set_text_content(Some("Hello, WebAssembly!"));

    // Add it to the page.
    let body = document.body().expect("document should have a body");
    body.append_child(&heading)
        .expect("failed to append heading");
}
