pub mod app;
pub use app::MainApp;

pub mod converter;

pub const WORK_UTILS_API_URL: &str = "https://work-utils-api.wyattverchere.com/";

pub const WSOL_ACCOUNT: &str = "So11111111111111111111111111111111111111112";
pub const WYATT_TEST_ACCOUNT: &str = "9czTJGaFdT863zno3r4t1Zd5j7pUePMguftxhm4piYwu";
pub const VYBE_TOKEN_ACCOUNT: &str = "vybe5DgwzGdvJMi4oH7TiQpubJd4QSDuGmbvWfACeb8";
pub const VYBE_STAKE_VALIDATOR: &str = "6oscGUEkXE8fyWoC4czRKbM1cuLkJNtgRsX1Un6w88Vf";

pub const GENERIC_BIG_ACCOUNTS: [&str; 9] = [
    "WLHv2UAZm6z4KyaaELi5pjdbJh6RESMva1Rnn8pJVVh", // 1.7m-ish
    "1BWutmTvYPwDtmw9abTkS4Ssr8no61spGAvW1X6NDix", // 1.4m-ish
    "STEPNwUmvdCWRm4yzH4rtCuPUeKuEapFvFKHKteiGH5", // 150k-ish
    "STEPNq2UGeGSzCyGVr2nMQAzf8xuejwqebd84wcksCK", // 100k-ish
    "DfMxre4cKmvogbLrPigxmibVTTQDuzjdXojWzjCXXhzj", // 35k-ish
    "1nc1nerator11111111111111111111111111111111", // 25k-ish
    "HUpPyLU8KWisCAr3mzWy2FKT6uuxQ2qGgJQxyTpDoes5", // 10k-ish
    "suqh5sHtr8HyJ7q8scBimULPkPpA557prMG47xCHQfK", // 10k-ish
    "8zFZHuSRuDpuAR7J6FzwyF3vKNx4CVW3DFHJerQhc7Zd", // 10k-ish
];

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
mod web;

#[cfg(all(target_arch = "wasm32", feature = "web_app"))]
pub use web::*;

lazy_static::lazy_static! {
    pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::ClientBuilder::new()
        .build()
        .expect("Failed to build reqwest client");
}
