use std::{env, io::Read};
use tiny_http::{Server, Response, Method, Header};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct StripePI {
    client_secret: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let secret_key = env::var("STRIPE_SECRET_KEY")
        .expect("Set STRIPE_SECRET_KEY in your environment");

    let port = env::var("MOCK_STRIPE_SERVER_PORT").unwrap_or_else(|_| "2718".to_string());
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)?;
    println!("Running on http://{}", addr);

    // Build CORS headers once
    let cors_headers = || {
        vec![
            Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(),
            Header::from_bytes("Access-Control-Allow-Methods", "POST, OPTIONS").unwrap(),
            Header::from_bytes("Access-Control-Allow-Headers", "Content-Type").unwrap(),
        ]
    };

    for mut request in server.incoming_requests() {
        // 1) Preflight
        if request.method() == &Method::Options {
            let mut resp = Response::empty(204);
            for h in cors_headers() {
                resp.add_header(h);
            }
            request.respond(resp)?;
            continue;
        }

        // 2) Routes
        match (request.method(), request.url()) {
            // Create Payment Intent
            (&Method::Post, "/create-payment-intent") => {
                // Read JSON body to get the "amount"
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;
                let amount: u32 = match serde_json::from_str::<serde_json::Value>(&body)
                    .ok()
                    .and_then(|v| v.get("amount").and_then(|a| a.as_u64()))
                {
                    Some(v) => v as u32,
                    None => {
                        let mut resp = Response::from_string("Invalid request: amount required").with_status_code(400);
                        for h in cors_headers() {
                            resp.add_header(h);
                        }
                        request.respond(resp)?;
                        return Ok(());
                    }
                };
            
                let client = reqwest::blocking::Client::new();
                let stripe_res = client
                    .post("https://api.stripe.com/v1/payment_intents")
                    .basic_auth(&secret_key, Some(""))
                    .form(&[
                        ("amount", amount.to_string()),
                        ("currency", "usd".to_string()),
                        ("expand[]", "charges.data.payment_method_details".to_string()),
                    ])
                    .send()?
                    .error_for_status()?
                    .json::<StripePI>()?;
            
                let body = json!({ "client_secret": stripe_res.client_secret }).to_string();
                let mut resp = Response::from_string(body)
                    .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
                for h in cors_headers() {
                    resp.add_header(h);
                }
                request.respond(resp)?;
            }
            
            
            // Webhook endpoint
            (&Method::Post, "/webhook") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;
                println!("Received webhook: {}", body);

                let mut resp = Response::from_string("OK");
                for h in cors_headers() {
                    resp.add_header(h);
                }
                request.respond(resp)?;
            }

            // 404 fallback
            _ => {
                let mut resp = Response::from_string("Not Found").with_status_code(404);
                for h in cors_headers() {
                    resp.add_header(h);
                }
                request.respond(resp)?;
            }
        }
    }

    // <-- make the Result<> happy
    Ok(())
}
