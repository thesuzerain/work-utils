pub mod app;
pub use app::MainApp;

pub mod converter;

pub const WORK_UTILS_API_URL : &str = "https://work-utils-api.wyattverchere.com/";

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
mod web;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
pub use web::*;

lazy_static::lazy_static! {
    pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .build()
        .expect("Failed to build reqwest client");
}