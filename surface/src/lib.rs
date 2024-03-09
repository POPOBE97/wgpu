use std::default;

use app_surface::{AppSurface, SurfaceFrame};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

use winit::{
  dpi::PhysicalSize, event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder
};

struct State {
  app: AppSurface,
}

impl State {
  fn new(app: AppSurface) -> Self {
    Self { app }
  }

  fn get_adapter_info(&self) -> wgpu::AdapterInfo {
    self.app.adapter.get_info()
  }

  fn resize(&mut self, size: &PhysicalSize<u32>) {
    if size.width == 0 || size.height == 0 { return };
    
    let pixel_width = ((size.width as f64) / self.app.get_view().scale_factor()).round() as u32;
    let pixel_height = ((size.height as f64) / self.app.get_view().scale_factor()).round() as u32;

    log::info!("[resize]: pixel_width {} pixel_height {}", pixel_width, pixel_height);

    if self.app.config.width == pixel_width && self.app.config.height == pixel_height {
      return;
    }
    self.app.resize_surface();
  }

  fn request_redraw(&mut self) {
    self.app.get_view().request_redraw();
  }

  fn render(&mut self) -> Result<(), wgpu::SurfaceError> {

    let (output, view) = self.app.get_current_frame_view(Some(self.app.config.format.add_srgb_suffix()));

    let mut encoder = self.app.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder")
      }
    );
    
    {
      let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("First Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment{
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations { 
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.1, g: 0.2, b: 0.3, a: 1.0
            }), 
            store: wgpu::StoreOp::Store
          }
        })],
        ..Default::default()
      });
    }

    self.app.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
  cfg_if::cfg_if! {
    if #[cfg(target_arch="wasm32")] {
      std::panic::set_hook(Box::new(console_error_panic_hook::hook));
      console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
    } else {
      // init log to print info to stdout
      std::env::set_var("RUST_LOG", "info");
      env_logger::init();
    }
  }

  let event_loop = EventLoop::new().unwrap();

  let window = WindowBuilder::new().build(&event_loop).unwrap();
  window.set_title("OpenGL Perf");

  // add canvas to the HTML document that we will host our application
  #[cfg(target_arch = "wasm32")]
  {
    log::info!("[run]: initializing html canvas");
    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    use winit::dpi::PhysicalSize;
    // let _ = window.request_inner_size(PhysicalSize::new(100, 400));

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
      .and_then(|win| win.document())
      .and_then(|doc| {
        let dst = doc.get_element_by_id("wgpu-container")?;
        let canvas = window.canvas().unwrap();
        canvas.set_width((100f32 * window.scale_factor() as f32) as u32);
        canvas.set_height((100f32 * window.scale_factor() as f32) as u32);
        let el = web_sys::HtmlCanvasElement::from(canvas);
        dst.append_child(&el).ok()?;
        Some(())
      })
      .expect("couldn't add canvas to document");
  }

  let app = app_surface::AppSurface::new(window).await;

  let mut state = State::new(app);

  let adapter_info = state.get_adapter_info();

  log::info!("[run]: adapter_info {:?}", adapter_info);

  event_loop.set_control_flow(ControlFlow::Wait);

  let _ = event_loop.run(move |event, control_flow| {
    match event {
      Event::WindowEvent {
        ref event,
        window_id,
      } if window_id == state.app.get_view().id() => {
        match event {
          WindowEvent::CloseRequested => control_flow.exit(),
          WindowEvent::Resized(new_size) => state.resize(new_size),
          WindowEvent::RedrawRequested => {
            match state.render() {
              Ok(_) => {}
              // Reconfigure the surface is lost
              Err(wgpu::SurfaceError::Lost) => log::error!("Surface is lost"),
              // The system is out of memory
              Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
              // others
              Err(e) => log::error!("{:?}", e),
            }
            state.request_redraw();
          }
          _ => {}
        }
      }
      _ => (),
    }
  });
}
