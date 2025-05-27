use std::{env, io::Read};
use tiny_http::{Server, Response, Method, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct CreateRequest {
    amount: u32,
    product: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct StripePI {
    client_secret: String,
    amount: Option<u32>,
    currency: Option<String>,
    charges: Option<StripeCharges>,
}

#[derive(Deserialize)]
struct StripeCharges {
    data: Vec<StripeCharge>,
}

#[derive(Deserialize)]
struct StripeCharge {
    receipt_url: Option<String>,
    status: Option<String>,
    payment_method_details: Option<StripePaymentMethodDetails>,
    outcome: Option<StripeOutcome>,
}

#[derive(Deserialize)]
struct StripePaymentMethodDetails {
    card: Option<StripeCard>,
}

#[derive(Deserialize)]
struct StripeCard {
    brand: Option<String>,
    last4: Option<String>,
}

#[derive(Deserialize)]
struct StripeOutcome {
    seller_message: Option<String>,
    network_status: Option<String>,
    reason: Option<String>,
    r#type: Option<String>,
}

#[derive(Serialize)]
struct CreateResponse {
    client_secret: String,
    amount: u32,
    currency: String,
    product: Option<String>,
    description: Option<String>,
    last4: Option<String>,
    brand: Option<String>,
    receipt_url: Option<String>,
    charge_status: Option<String>,
    outcome: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let secret_key = env::var("STRIPE_SECRET_KEY")
        .expect("Set STRIPE_SECRET_KEY in your environment");

    let port = env::var("MOCK_STRIPE_SERVER_PORT").unwrap_or_else(|_| "2718".to_string());
    let addr = format!("127.0.0.1:{}", port);
    let server = Server::http(&addr)?;
    println!("Running on http://{}", addr);

    let cors_headers = || {
        vec![
            Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap(),
            Header::from_bytes("Access-Control-Allow-Methods", "POST, OPTIONS").unwrap(),
            Header::from_bytes("Access-Control-Allow-Headers", "Content-Type").unwrap(),
        ]
    };

    for mut request in server.incoming_requests() {
        if request.method() == &Method::Options {
            let mut resp = Response::empty(204);
            for h in cors_headers() {
                resp.add_header(h);
            }
            request.respond(resp)?;
            continue;
        }

        match (request.method(), request.url()) {
            (&Method::Post, "/create-payment-intent") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body)?;

                let parsed: CreateRequest = match serde_json::from_str(&body) {
                    Ok(val) => val,
                    Err(_) => {
                        let mut resp = Response::from_string("Invalid request: amount required")
                            .with_status_code(400);
                        for h in cors_headers() {
                            resp.add_header(h);
                        }
                        request.respond(resp)?;
                        continue;
                    }
                };

                let amount = parsed.amount;
                let product = parsed.product.clone();
                let description = parsed.description.clone();

                let client = reqwest::blocking::Client::new();
                let stripe_res = client
                    .post("https://api.stripe.com/v1/payment_intents")
                    .basic_auth(&secret_key, Some(""))
                    .form(&[
                        ("amount", amount.to_string()),
                        ("currency", "usd".to_string()),
                        // "expand" gets full charge/card/receipt details
                        ("expand[]", "charges.data.payment_method_details".to_string()),
                        ("expand[]", "charges.data.outcome".to_string()),
                    ])
                    .send()?
                    .error_for_status()?
                    .json::<StripePI>()?;

                let mut last4 = None;
                let mut brand = None;
                let mut receipt_url = None;
                let mut charge_status = None;
                let mut outcome = None;

                if let Some(charges) = &stripe_res.charges {
                    if let Some(charge) = charges.data.get(0) {
                        if let Some(ref details) = charge.payment_method_details {
                            if let Some(ref card) = details.card {
                                last4 = card.last4.clone();
                                brand = card.brand.clone();
                            }
                        }
                        receipt_url = charge.receipt_url.clone();
                        charge_status = charge.status.clone();
                        if let Some(ref out) = charge.outcome {
                            outcome = out.seller_message.clone();
                        }
                    }
                }

                let resp_obj = CreateResponse {
                    client_secret: stripe_res.client_secret,
                    amount: stripe_res.amount.unwrap_or(amount),
                    currency: stripe_res.currency.unwrap_or_else(|| "usd".to_string()),
                    product,
                    description,
                    last4,
                    brand,
                    receipt_url,
                    charge_status,
                    outcome,
                };

                let body = serde_json::to_string(&resp_obj).unwrap();
                let mut resp = Response::from_string(body)
                    .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
                for h in cors_headers() {
                    resp.add_header(h);
                }
                request.respond(resp)?;
            }

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

            _ => {
                let mut resp = Response::from_string("Not Found").with_status_code(404);
                for h in cors_headers() {
                    resp.add_header(h);
                }
                request.respond(resp)?;
            }
        }
    }

    Ok(())
}
