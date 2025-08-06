pub mod app;
pub use app::MainApp;

pub mod converter;

pub const WORK_UTILS_API_URL: &str = "https://work-utils-api.wyattverchere.com/";

pub const WSOL_ACCOUNT: &str = "So11111111111111111111111111111111111111112";
pub const WYATT_TEST_ACCOUNT: &str = "9czTJGaFdT863zno3r4t1Zd5j7pUePMguftxhm4piYwu";
pub const VYBE_TOKEN_ACCOUNT: &str = "vybe5DgwzGdvJMi4oH7TiQpubJd4QSDuGmbvWfACeb8";
pub const VYBE_STAKE_VALIDATOR: &str = "6oscGUEkXE8fyWoC4czRKbM1cuLkJNtgRsX1Un6w88Vf";

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
mod web;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
pub use web::*;

lazy_static::lazy_static! {
    pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .build()
        .expect("Failed to build reqwest client");
}
