use surface::run;

fn main() {
  #[cfg(not(target_arch = "wasm32"))]
  tokio::runtime::Runtime::new().unwrap().block_on(run());
}