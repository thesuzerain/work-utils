pub mod app;
pub use app::MainApp;

pub mod bytes_base58;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;
