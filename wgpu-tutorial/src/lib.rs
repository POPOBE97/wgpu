use winit::{
  dpi::PhysicalSize, event::*, event_loop::{ControlFlow, EventLoop}, window::{self, Window, WindowBuilder}
};

use wgpu::{Surface, SurfaceTargetUnsafe};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

struct State {
  surface: wgpu::Surface<'static>,
  device: wgpu::Device,
  queue: wgpu::Queue,
  config: wgpu::SurfaceConfiguration,
  size: winit::dpi::PhysicalSize<u32>,
  // The window must be declared after the surface so
  // it gets dropped after it as the surface contains
  // unsafe references to the window's resources.
  window: Window,
}

impl State {
    async fn new(window: Window) -> Self {
      let size = window.inner_size();
      // let size = PhysicalSize::new(100, 100);

      // The instance is a handler to the GPU
      // Backend::all => Vulkan + Metal + DX12 + Browser WebGPU
      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
      });

      // # safety
      cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
          use winit::platform::web::WindowExtWebSys;
          let canvas = window.canvas().unwrap();
          let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas)).unwrap();
        } else {
          let surface = unsafe { instance.create_surface_unsafe(SurfaceTargetUnsafe::from_window(&window).unwrap()) }.unwrap();
        }
      }

      let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
      }).await.unwrap();

      let adapter_info = adapter.get_info();
      let gpu_info = format!(
        "using {}, backend {:?}。",
        adapter_info.name, adapter_info.backend
      );

      #[cfg(not(target_arch = "wasm32"))]
      println!("{gpu_info}");
      #[cfg(target_arch = "wasm32")]
      log::info!( "{gpu_info}" );

      let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
          label: None,
          required_features: wgpu::Features::empty(),
          required_limits: {
            cfg_if::cfg_if! {
              if #[cfg(all(target_arch="wasm32", feature="webgl"))] {
                wgpu::Limits::downlevel_webgl2_defaults()
              } else {
                wgpu::Limits::default()
              }
            }
          },
        },
        None
      ).await.unwrap();
      
      let surface_caps = surface.get_capabilities(&adapter);

      // use sRGB if available
      let prefered = surface_caps.formats[0];
      let surface_format = if cfg!(all(target_arch = "wasm32", not(feature = "webgl"))) {
        // Chrome WebGPU doesn't support sRGB:
        // unsupported swap chain format "xxxx8unorm-srgb"
          prefered.remove_srgb_suffix()
      } else {
          prefered
      };

      let view_formats = if cfg!(feature = "webgl") {
        // panicked at 'Error in Surface::configure: Validation Error
        // Caused by:
        // Downlevel flags DownlevelFlags(SURFACE_VIEW_FORMATS) are required but not supported on the device.
        vec![]
      } else {
        vec![surface_format.add_srgb_suffix(), surface_format.remove_srgb_suffix()]
      };

      let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: view_formats,
        desired_maximum_frame_latency: 2,
      };
      surface.configure(&device, &config);

      Self { surface, device, queue, config, size, window }
    }

    pub fn window(&self) -> &Window {
      &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
      if new_size.width > 0 && new_size.height > 0 {
        self.size = new_size;
        #[cfg(target_arch="wasm32")] {
          let size_info = format!("[resize]: w {} h {}", new_size.width, new_size.height);
          log::info!("{size_info}");
        }
        self.config.width = ((new_size.width as f64) / self.window().scale_factor()) as u32;
        self.config.height = ((new_size.height as f64) / self.window().scale_factor()) as u32;
        self.surface.configure(&self.device, &self.config)
      }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
      false
    }

    fn update(&mut self) { }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
      let output = self.surface.get_current_texture()?;
      let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

      let mut encoder = self.device.create_command_encoder(& wgpu::CommandEncoderDescriptor{
        label: Some("Render Encoder"),
      });

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
          depth_stencil_attachment: None,
          occlusion_query_set: None,
          timestamp_writes: None,
        });
      };

      self.queue.submit(std::iter::once(encoder.finish()));
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
      env_logger::init();
    }
  }
  
  let event_loop = EventLoop::new().unwrap();

  let window = WindowBuilder::new().build(&event_loop).unwrap();
  window.set_title("Rust GPU Programming");

  // add canvas to the HTML document that we will host our application
  #[cfg(target_arch="wasm32")] {
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

  let mut state = State::new(window).await;

  event_loop.set_control_flow(ControlFlow::Wait);
  
  let _ = event_loop.run(move |event, control_flow| {
    match event {
      Event::WindowEvent { 
        ref event, 
        window_id 
      } if window_id == state.window().id() =>  if !state.input(event) {
        match event {
          WindowEvent::CloseRequested => control_flow.exit(),
          WindowEvent::Resized(new_size) => state.resize(*new_size),
          WindowEvent::RedrawRequested => {
            state.update();
            match state.render() {
              Ok(_) => {},
              // Reconfigure the surface is lost
              Err(wgpu::SurfaceError::Lost) => eprintln!("Surface is lost"),
              // The system is out of memory
              Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
              //  others
              Err(e) => eprintln!("{:?}", e),
            }
          },
          _ => {}
        }
      },

      Event::AboutToWait => {
        // RedrawRequested will only trigger once unless we manually request it.
        state.window().request_redraw();
      },

      _ => (),
    }
  });

}