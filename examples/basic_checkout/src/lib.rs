//! yew_stripe/examples/basic_checkout/src/lib.rs
//!
//! Demonstration of a simple shop with 3 products
//! and a variety of credit cards for you to buy them with.

use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_stripe::{client::StripeError, StripeCheckout, StripeCheckoutSuccess};


/// Response from our backend when creating a PaymentIntent.
/// 
/// **Configurable**: your server should return a JSON object matching this shape, or you should update this to match your server's response type.
#[derive(Deserialize)]
struct CreatePIResponse {
    /// The PaymentIntent client secret, used by StripeCheckout to confirm the payment.
    client_secret: String,
}

/// A product available for purchase in this demo store.
#[derive(Clone, PartialEq)]
struct Product {
    id: usize,
    name: &'static str,
    description: &'static str,
    price: u32, // in cents
}

/// Static list of products. Change this to ship your own catalog.
const PRODUCTS: &[Product] = &[
    Product {
        id: 1,
        name: "Cap",
        description: "A stylish cap to keep the sun away. Great for adventures and weekends. ☀️",
        price: 500,
    },
    Product {
        id: 2,
        name: "T-Shirt",
        description: "Soft, comfy, and goes with anything. The classic tee for every day. 👕",
        price: 2900,
    },
    Product {
        id: 3,
        name: "Shoes",
        description: "Run faster with these sneakers. Comfort meets style. 🏃",
        price: 11300,
    },
];

/// Application view state: product list or checkout page.
#[derive(PartialEq)]
enum AppView {
    ProductList,
    Checkout { product: Product },
}

/// Entry point for the WASM module.
#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}

/// Root component managing view state.
#[function_component(App)]
fn app() -> Html {
    let view = use_state(|| AppView::ProductList);

    match &*view {
        AppView::ProductList => {
            html! {
                <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-200 flex flex-col items-center py-10">
                    <h1 class="text-4xl font-extrabold mb-8 text-gray-800 drop-shadow-sm tracking-tight">{"Yew Shop"}</h1>
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

/// Props for the checkout page component.
#[derive(Properties, PartialEq, Clone)]
struct CheckoutPageProps {
    /// The product being purchased.
    product: Product,
    /// Callback to navigate back to the product list.
    on_back: Callback<()>,
}

/// Checkout page: fetches a client secret and renders StripeCheckout.
///
/// - Updates when `props.product` changes.
/// - **Configurable**: adjust `backend` URL below to point at your server.
#[function_component(CheckoutPage)]
fn checkout_page(props: &CheckoutPageProps) -> Html {
    let client_secret = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let paid = use_state(|| None::<StripeCheckoutSuccess>);

    // Fetch client_secret for this product price on mount
    {
        let client_secret = client_secret.clone();
        let error = error.clone();
        let product = props.product.clone();
        use_effect_with(product.clone(), move |product| {
            let client_secret = client_secret.clone();
            let error = error.clone();
            let product = product.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let backend = "http://127.0.0.1:2718/create-payment-intent";

                let req = Request::post(backend)
                    .header("Content-Type", "application/json")
                    .body(format!(r#"{{"amount":{}}}"#, product.price))
                    .unwrap();
                let resp = req.send().await;
                match resp {
                    Ok(r) if r.ok() => {
                        let res = r.json::<CreatePIResponse>().await;
                        match res {
                            Ok(data) => client_secret.set(Some(data.client_secret)),
                            Err(e) => error.set(Some(format!("Bad JSON: {}", e))),
                        }
                    }
                    Ok(r) => error.set(Some(format!("Server error: {}", r.status()))),
                    Err(e) => error.set(Some(format!("Network error: {}", e))),
                }
            });
            || ()
        });
    }

    // on_success callback for StripeCheckout
    let on_success = {
        let paid = paid.clone();
        Callback::from(move |s: StripeCheckoutSuccess| {
            paid.set(Some(s));
        })
    };

    // on_error callback for StripeCheckout
    let on_error = {
        let error = error.clone();
        Callback::from(move |err: StripeError| {
            error.set(Some(err.message));
        })
    };

    // Handler to go back to product list
    let on_back = {
        let on_back = props.on_back.clone();
        Callback::from(move |_| on_back.emit(()))
    };

    html! {
        <div class="min-h-screen bg-slate-50 flex flex-col items-center pt-8 px-2">
            <div class="w-full max-w-4xl flex flex-col items-center">
                <div class="w-full max-w-2xl bg-white rounded-2xl shadow-lg p-8 flex flex-col mb-8 border border-slate-100">
                    <button onclick={on_back}
                        class="mb-4 text-blue-500 hover:text-blue-700 font-medium text-sm text-left"
                        aria-label="Back to product list">
                        {"← Back"}
                    </button>
                    <h2 class="text-2xl font-bold mb-1 text-gray-800 tracking-tight">{ &props.product.name }</h2>
                    <p class="mb-2 text-gray-600 text-base">{ &props.product.description }</p>
                    <div class="mb-6 text-2xl font-extrabold text-blue-700 tracking-tight">
                        { format!("${:.2}", props.product.price as f32 / 100.0) }
                    </div>
                    {
                        if let Some(success) = &*paid {
                            html! {
                                <div class="rounded-lg bg-green-50 p-4 shadow-inner flex flex-col items-center">
                                    <div class="text-green-700 text-lg font-semibold mb-2">{"✅ Payment Successful"}</div>
                                    <div class="text-gray-900 text-xl font-bold mb-1">{ &props.product.name }</div>
                                    <div class="text-gray-600 mb-4">{ &props.product.description }</div>
                                    <div class="text-green-700 text-base font-bold">{ format!("You paid ${:.2}", props.product.price as f32 / 100.0) }</div>
                                    {
                                        if let (Some(brand), Some(last4)) = (&success.brand, &success.last4) {
                                            html!{ <div class="text-gray-700 text-base mb-1">{ format!("Card: {} ending in {}", brand, last4) }</div> }
                                        } else {
                                            Html::default()
                                        }
                                    }
                                    {
                                        if let Some(url) = &success.receipt_url {
                                            html!{
                                                <a href={url.clone()} target="_blank"
                                                   class="text-blue-600 underline text-sm mt-2">
                                                    {"View receipt"}
                                                </a>
                                            }
                                        } else {
                                            Html::default()
                                        }
                                    }
                                </div>
                            }
                        } else if let Some(cs) = &*client_secret {
                            html!{
                                <StripeCheckout
                                    publishable_key={"pk_test_51KUI60DEw04PTNScWne4kC3RDrpxnydTfgx0B4b4EsBJajLDmqT2t79nEj8kZjeMGx2bfI9BZN1zqo2NX6HrGp4u00Rv0S1OYT".to_string()}
                                    client_secret={cs.clone()}
                                    button_label={format!("Pay ${:.2}", props.product.price as f32 / 100.0)}
                                    on_success={on_success}
                                    on_error={on_error}
                                />
                            }
                        } else if let Some(e) = &*error {
                            html!{ <div class="text-red-600 font-semibold">{ e }</div> }
                        } else {
                            html!{ <div class="text-slate-500">{ "Loading checkout…" }</div> }
                        }
                    }
                </div>
                <div class="my-6 h-px bg-slate-200 w-full" />
                <div class="mb-4 w-full flex flex-col items-center">
                    <div class="text-xs uppercase tracking-wider font-semibold text-slate-400 mb-2">
                        {"Test Card Numbers (For Testing Purposes Only)"}
                    </div>
                </div>
                <TestCardReference />
            </div>
        </div>
    }
}

/// Renders tables of valid and invalid test card numbers.
///
/// **Useful for testing**: no need to memorize real card data here.
#[function_component(TestCardReference)]
pub fn test_card_reference() -> Html {
    // Lists of test card data; update if Stripe changes test numbers.
    let valid_cards = vec![
        (
            "Visa",
            "4242 4242 4242 4242",
            "Any 3 digits",
            "Any future date",
        ),
        (
            "Mastercard",
            "5555 5555 5555 4444",
            "Any 3 digits",
            "Any future date",
        ),
        (
            "American Express",
            "3782 822463 10005",
            "Any 4 digits",
            "Any future date",
        ),
        (
            "Discover",
            "6011 1111 1111 1117",
            "Any 3 digits",
            "Any future date",
        ),
    ];
    let invalid_cards = vec![
        (
            "Insufficient Funds",
            "4000 0000 0000 9995",
            "Any 3 digits",
            "Any future date",
            "card_declined: insufficient_funds",
        ),
        (
            "Lost Card",
            "4000 0000 0000 9987",
            "Any 3 digits",
            "Any future date",
            "card_declined: lost_card",
        ),
        (
            "Expired Card",
            "4000 0000 0000 0069",
            "Any 3 digits",
            "Expired date",
            "expired_card",
        ),
        (
            "Incorrect CVC",
            "4000 0000 0000 0127",
            "Wrong 3 digits",
            "Any future date",
            "incorrect_cvc",
        ),
    ];

    html! {
        <div class="w-full max-w-4xl mx-auto flex flex-col md:flex-row gap-8 my-8">
            <div class="flex-1 bg-white rounded-xl shadow-md border border-slate-100 p-6">
                <h3 class="text-lg font-bold text-green-700 mb-4">{"Valid Test Cards"}</h3>
                <table class="w-full text-sm table-auto">
                    <thead>
                        <tr class="text-slate-500 border-b">
                            <th class="text-left pb-2">{"Brand"}</th>
                            <th class="text-left pb-2">{"Card Number"}</th>
                            <th class="text-left pb-2">{"CVC"}</th>
                            <th class="text-left pb-2">{"Exp."}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for valid_cards.iter().map(|(brand, number, cvc, exp)| html! {
                            <tr class="hover:bg-slate-50 group cursor-pointer select-all">
                                <td class="py-1 font-semibold text-slate-700">{ brand }</td>
                                <td class="py-1 tabular-nums text-slate-800">{ number }</td>
                                <td class="py-1">{ cvc }</td>
                                <td class="py-1">{ exp }</td>
                            </tr>
                        }) }
                    </tbody>
                </table>
                <div class="text-xs text-slate-400 mt-2">{"Click any value to copy. Use any future date."}</div>
            </div>
            <div class="flex-1 bg-white rounded-xl shadow-md border border-slate-100 p-6">
                <h3 class="text-lg font-bold text-red-700 mb-4">{"Invalid Test Cards"}</h3>
                <table class="w-full text-sm table-auto">
                    <thead>
                        <tr class="text-slate-500 border-b">
                            <th class="text-left pb-2">{"Scenario"}</th>
                            <th class="text-left pb-2">{"Card Number"}</th>
                            <th class="text-left pb-2">{"CVC"}</th>
                            <th class="text-left pb-2">{"Exp."}</th>
                            <th class="text-left pb-2">{"Error"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for invalid_cards.iter().map(|(scenario, number, cvc, exp, err)| html! {
                            <tr class="hover:bg-slate-50 group cursor-pointer select-all">
                                <td class="py-1 font-semibold text-slate-700">{ scenario }</td>
                                <td class="py-1 tabular-nums text-slate-800">{ number }</td>
                                <td class="py-1">{ cvc }</td>
                                <td class="py-1">{ exp }</td>
                                <td class="py-1 text-xs text-slate-400">{ err }</td>
                            </tr>
                        }) }
                    </tbody>
                </table>
                <div class="text-xs text-slate-400 mt-2">{"Click any value to copy. Use any future date unless noted."}</div>
            </div>
        </div>
    }
}
