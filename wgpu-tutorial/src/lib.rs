use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
  cfg_if::cfg_if! {
    if #[cfg(target_arch="wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
      console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    } else {
      env_logger::init();
    }
  }

  // add canvas to the HTML document that we will host our application
  #[cfg(target_arch="wasm32")] {
    use winit::dpi::PhysicalSize;
    window.set_inner_size(PhysicalSize::new(450, 400));

    use winit::platform::web::WindowBuilderExtWebSys;
    web_sys::window()
      .and_then(|win| win.document())
      .and_then(|doc| {
        let dst = doc.get_element_by_id("wgpu-canvas").unwrap();
        let canvas = web_sys::Element::from(window.canvas());
        dst.append_child(&canvas).ok();
        Some(())
      })
      .expect("couldn't add canvas to document");
  }

  let event_loop = EventLoop::new().unwrap();
  let window = WindowBuilder::new().build(&event_loop).unwrap();
  window.set_title("Rust GPU Programming");

  event_loop.set_control_flow(ControlFlow::Wait);
  let _ = event_loop.run(move |event, control_flow| {
    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        println!("The close button was pressed; stopping");
        control_flow.exit();
      },
      Event::AboutToWait => {
        window.request_redraw();
      },
      Event::WindowEvent {
        event: WindowEvent::RedrawRequested,
        ..
      } => {
        // println!("Redraw requested");
      },
      _ => (),
    }
  });

}


#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn main() {
  run();
}