// src/lib.rs

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use yew::prelude::*;
use yew_stripe::client::{
    ElementsOptions, ConfirmPaymentParams, mount_payment_element, confirm_payment, PaymentResult,
};
use yew_stripe::use_stripejs;
use gloo_net::http::Request;
use serde::Deserialize;
use serde_json::Value;
use web_sys::js_sys;
use js_sys::{Reflect, Promise, Function}; // ← use js_sys (should be in your dependencies)
use wasm_bindgen::JsCast;
use gloo_utils::format::JsValueSerdeExt;

#[derive(Deserialize)]
struct CreatePIResponse {
    client_secret: String,
}

#[derive(Clone, PartialEq)]
struct Product {
    id: usize,
    name: &'static str,
    description: &'static str,
    price: u32, // in cents
}

const PRODUCTS: &[Product] = &[
    Product { id: 1, name: "Cap",     description: "A stylish cap to keep the sun away. Great for adventures and weekends. ☀️",    price: 500 },
    Product { id: 2, name: "T-Shirt", description: "Soft, comfy, and goes with anything. The classic tee for every day. 👕",        price: 2900 },
    Product { id: 3, name: "Shoes",   description: "Run faster with these sneakers. Comfort meets style. 🏃",                      price: 11300 },
];

#[derive(PartialEq)]
enum AppView {
    ProductList,
    Checkout { product: Product },
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let view = use_state(|| AppView::ProductList);

    match &*view {
        AppView::ProductList => {
            html! {
                <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-200 flex flex-col items-center py-10">
                    <h1 class="text-4xl font-extrabold mb-8 text-gray-800 drop-shadow-sm tracking-tight">{"Mock Store"}</h1>
                    <h2 class="text-5xl font-extrabold mb-8 text-gray-800 drop-shadow-sm tracking-tight">{"Powered by Stripe"}</h2>
                    <p class="mb-8 text-gray-500 text-lg font-medium">{"Select a product to check out securely."}</p>
                    <div class="w-full max-w-2xl grid grid-cols-1 md:grid-cols-3 gap-8">
                        { for PRODUCTS.iter().map(|p| {
                            let view = view.clone();
                            let p2 = p.clone();
                            let click = Callback::from(move |_| view.set(AppView::Checkout { product: p2.clone() }));
                            html! {
                                <div class="bg-white rounded-2xl shadow-md hover:shadow-xl transition-shadow p-6 flex flex-col items-center border border-slate-100">
                                    <h2 class="text-xl font-bold mb-1 text-gray-700">{ p.name }</h2>
                                    <p class="mb-2 text-gray-500 text-center">{ p.description }</p>
                                    <div class="mb-4 text-lg font-semibold text-blue-700">{ format!("${:.2}", p.price as f32 / 100.0) }</div>
                                    <button onclick={click}
                                            class="mt-auto px-4 py-2 bg-blue-600 text-white rounded font-semibold shadow-sm hover:bg-blue-700 focus:ring-2 focus:ring-blue-400 focus:outline-none transition"
                                            aria-label={format!("Buy {}", p.name)}>
                                        {"Buy Now"}
                                    </button>
                                </div>
                            }
                        })}
                    </div>
                </div>
            }
        }
        AppView::Checkout { product } => {
            html! {
                <CheckoutPage product={product.clone()} on_back={Callback::from({
                    let view = view.clone();
                    move |_| view.set(AppView::ProductList)
                })}/>
            }
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
struct CheckoutPageProps {
    product: Product,
    on_back: Callback<()>,
}

#[function_component(CheckoutPage)]
fn checkout_page(props: &CheckoutPageProps) -> Html {
    let stripe_ready   = use_stripejs();
    let stripe_el      = use_mut_ref(|| None::<(JsValue, JsValue)>);
    let client_secret  = use_state(|| String::new());
    let error          = use_state(|| None::<String>);
    let loading        = use_state(|| false);
    let success        = use_state(|| None::<(f64, String)>);
    let requested_amt  = props.product.price;

    // Fetch client_secret for this product & mount Payment Element
    {
        let error = error.clone();
        let stripe_el = stripe_el.clone();
        let client_secret = client_secret.clone();
        let ready = stripe_ready;
        let amount = requested_amt;
        use_effect_with((stripe_ready.clone(), requested_amt), {
            let error = error.clone();
            let stripe_el = stripe_el.clone();
            let client_secret = client_secret.clone();
            move |(stripe_ready, requested_amt)| {
                if *stripe_ready {
                    let error = error.clone();
                    let stripe_el = stripe_el.clone();
                    let client_secret = client_secret.clone();
                    let amt = *requested_amt;
                    spawn_local(async move {
                        let pk = "pk_test_51KUI60DEw04PTNScWne4kC3RDrpxnydTfgx0B4b4EsBJajLDmqT2t79nEj8kZjeMGx2bfI9BZN1zqo2NX6HrGp4u00Rv0S1OYT";
                        let backend = "http://127.0.0.1:2718/create-payment-intent";
                        let req = Request::post(backend)
                            .header("Content-Type", "application/json")
                            .body(format!(r#"{{"amount":{}}}"#, amt))
                            .unwrap();
                        let resp = req.send().await;
                        let cs = match resp {
                            Ok(r) if r.ok() => r.json::<CreatePIResponse>().await
                                .map(|d| d.client_secret)
                                .unwrap_or_else(|e| { error.set(Some(format!("Bad JSON: {}", e))); String::new() }),
                            Ok(r) => { error.set(Some(format!("Server error: {}", r.status()))); String::new() }
                            Err(e) => { error.set(Some(format!("Network error: {}", e))); String::new() }
                        };
                        if cs.is_empty() { return; }
                        client_secret.set(cs.clone());
        
                        let opts = ElementsOptions {
                            client_secret: cs.into(),
                            appearance: None,
                        };
                        match mount_payment_element(pk, opts, "#payment-element", None).await {
                            Ok((stripe, elements, _)) => {
                                *stripe_el.borrow_mut() = Some((stripe.into(), elements.into()));
                            }
                            Err(err) => error.set(Some(err.message)),
                        }
                    });
                }
                || ()
            }
        });
        
    }

    // Payment submit
    let on_click = {
        let error    = error.clone();
        let loading  = loading.clone();
        let success  = success.clone();
        let stripe_el = stripe_el.clone();
        let cs = (*client_secret).clone();
        Callback::from(move |_| {
            if *loading || success.is_some() { return; }
            if let Some((s, e)) = &*stripe_el.borrow() {
                let s = s.clone();
                let e = e.clone();
                let error = error.clone();
                let loading = loading.clone();
                let success = success.clone();
                let cs = cs.clone();
                spawn_local(async move {
                    loading.set(true);
                    error.set(None);
                    let params = ConfirmPaymentParams {
                        return_url: None,
                        save_payment_method: None,
                        extra: None,
                    };
                    match confirm_payment(&s.clone().into(), &e.into(), params, None, true).await {
                        PaymentResult::Success(_) => {
                            // retrieve full PaymentIntent
                            let stripe_js = s.into();
                            let fn_retrieve = Reflect::get(&stripe_js, &JsValue::from_str("retrievePaymentIntent"))
                                .expect("retrievePaymentIntent not found")
                                .unchecked_into::<Function>();
                            let promise: Promise = fn_retrieve
                                .call1(&stripe_js, &JsValue::from_str(&cs))
                                .unwrap()
                                .unchecked_into();
                            let result = JsFuture::from(promise).await.unwrap();
                            let pi_js = Reflect::get(&result, &JsValue::from_str("paymentIntent"))
                                .unwrap();
                            let pi_json: Value = pi_js.into_serde().unwrap_or_default();
                            let amt_cents = pi_json.get("amount_received")
                                .and_then(|v| v.as_i64())
                                .or_else(|| pi_json.get("amount").and_then(|v| v.as_i64()))
                                .unwrap_or(0);
                            let amount = amt_cents as f64 / 100.0;
                            let last4 = pi_json.get("charges")
                                .and_then(|c| c.get("data"))
                                .and_then(|d| d.as_array())
                                .and_then(|arr| arr.get(0))
                                .and_then(|first| first.get("payment_method_details"))
                                .and_then(|pmd| pmd.get("card"))
                                .and_then(|card| card.get("last4"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("<unknown>")
                                .to_string();
                            success.set(Some((amount, last4)));
                        }
                        PaymentResult::Error(err) => {
                            error.set(Some(err.message));
                        }
                    }
                    loading.set(false);
                });
            }
        })
    };

    let on_back = {
        let on_back = props.on_back.clone();
        Callback::from(move |_| on_back.emit(()))
    };

    html! {
        <div class="min-h-screen bg-slate-50 flex flex-col items-center pt-12 px-2">
            <div class="w-full max-w-md bg-white rounded-2xl shadow-md p-8 flex flex-col">
                <button onclick={on_back}
                        class="mb-4 text-blue-500 hover:text-blue-700 font-medium text-sm text-left"
                        aria-label="Back to product list">
                    {"← Back"}
                </button>
                <h2 class="text-2xl font-bold mb-1 text-gray-800 tracking-tight">{ props.product.name }</h2>
                <p class="mb-2 text-gray-600 text-base">{ props.product.description }</p>
                <div class="mb-6 text-xl font-semibold text-blue-700">
                    { format!("${:.2}", props.product.price as f32 / 100.0) }
                </div>
                {
                    if let Some((amt, last4)) = &*success {
                        html! {
                            <div class="rounded-lg bg-green-50 p-4 shadow-inner flex flex-col items-center">
                                <div class="text-green-700 text-lg font-semibold mb-2">{"✅ Payment Successful"}</div>
                                <div class="text-green-700 text-base">{ format!("You paid ${:.2}", amt) }</div>
                                <div class="text-green-700 text-sm">{ format!("Card ending in {}", last4) }</div>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                <div id="payment-element" class="mb-4"/>
                                <button onclick={on_click}
                                    disabled={!stripe_ready || *loading}
                                    class="w-full mt-2 py-2 rounded font-semibold shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:ring-2 focus:ring-blue-400 focus:outline-none transition disabled:opacity-60">
                                    {
                                        if *loading {
                                            "Processing…".to_string()
                                        } else {
                                            format!("Pay ${:.2}", props.product.price as f32 / 100.0)
                                        }
                                    }
                                </button>
                                {
                                    if let Some(msg) = &*error {
                                        html! { <p class="mt-3 text-sm text-red-600">{ msg.clone() }</p> }
                                    } else {
                                        Html::default()
                                    }
                                }
                            </>
                        }
                    }
                }
            </div>
        </div>
    }
}
