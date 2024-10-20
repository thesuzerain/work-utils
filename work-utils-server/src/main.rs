use std::time::Duration;

use actix_extensible_rate_limit::{backend::memory::InMemoryBackend, RateLimiter};
use actix_web::{get, http::StatusCode, web, App, HttpServer, Responder};
use serde_json::json;

#[actix_web::main]
async fn main() {
    let backend = InMemoryBackend::builder().build();

    HttpServer::new(move || {
        // Assign a limit of 5 requests per minute per client ip address
        let input = actix_extensible_rate_limit::backend::SimpleInputFunctionBuilder::new(
            Duration::from_secs(60),
            120,
        )
        .real_ip_key()
        .build();
        let rate_limit_middleware = RateLimiter::builder(backend.clone(), input)
            .add_headers()
            .build();

        App::new().wrap(rate_limit_middleware).service(greet)
    })
    .bind(("0.0.0.0", 8080))
    .expect("Failed to bind to port")
    .run()
    .await
    .expect("Failed to run server");
}

fn get_reqwest_client() -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .build()
        .expect("Failed to build reqwest client")
}

#[get("/solana_blocktime/{block}")]
async fn greet(block: web::Path<u32>) -> impl Responder {
    let client = get_reqwest_client();
    let block = block.into_inner();

    // Send a call to the Solana RPC API to get the block time
    let sent = match client
        .post("https://api.mainnet-beta.solana.com")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBlockTime",
            "params": [block]
        }))
        .send()
        .await
    {
        Ok(t) => t,
        Err(e) => {
            return (
                web::Json(json!(
                {
                    "error": format!("Failed to send request: {:?}", e)
                }
                )),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };

    // Parse response
    let response = match sent.json::<serde_json::Value>().await {
        Ok(t) => t,
        Err(e) => {
            return (
                web::Json(json!(
                    {
                        "error": format!("Failed to parse response: {:?}", e)
                    }
                )),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };

    // Fetch the block time from the response (one field, don't bother deserialiazing)
    let value = match response.get("result").map(|v| v.as_i64()).flatten() {
        Some(t) => t,
        None => {
            return (
                web::Json(json!(
                    {
                        "error": "Failed to get block time from response"
                    }
                )),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    };

    // Return the block time
    (
        web::Json(json!(
            {
                "block": block,
                "timestamp": value
            }
        )),
        StatusCode::OK,
    )
}
