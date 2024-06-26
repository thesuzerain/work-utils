pub mod app;
pub use app::MainApp;

pub mod converter;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
mod web;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
pub use web::*;
