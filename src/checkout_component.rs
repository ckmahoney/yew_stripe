//! A high-level Yew Stripe checkout component for your app to drop in.
//! Users get idiomatic Yew props and event callbacks.

use yew::prelude::*;
use crate::{
    JsStripe, JsElements, JsPaymentElement,
    client::{
        ElementsOptions, ConfirmPaymentParams, PaymentElementOptions, mount_payment_element, confirm_payment, PaymentResult, StripeError,
    }
};

// Needed for working with JsValue and conversions (trait imports).
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use web_sys::js_sys;
use gloo_utils::format::JsValueSerdeExt;

use crate::client::validate_payment_element;

use crate::use_stripejs;

// Define what you want to emit to parent
#[derive(Clone, PartialEq, Debug)]
pub struct StripeCheckoutSuccess {
    pub amount: f64,
    pub last4: Option<String>,
    pub brand: Option<String>,
    pub receipt_url: Option<String>,
    pub payment_intent_id: Option<String>,
}

#[derive(Properties, PartialEq, Clone)]
pub struct StripeCheckoutProps {
    pub publishable_key: String,
    pub client_secret: String,
    #[prop_or_default]
    pub payment_element_options: Option<PaymentElementOptions>,
    #[prop_or_default]
    pub on_success: Callback<StripeCheckoutSuccess>,
    #[prop_or_default]
    pub on_error: Callback<StripeError>,
    #[prop_or_default]
    pub button_label: Option<String>,
    #[prop_or_default]
    pub children: Children, // allow extra UI (product summary etc)
}

#[function_component(StripeCheckout)]
pub fn stripe_checkout(props: &StripeCheckoutProps) -> Html {
    let stripe_ready = use_stripejs();
    let state = use_state(|| None::<(JsStripe, JsElements, JsPaymentElement)>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    // Mount Stripe Payment Element on load
    {
        let state = state.clone();
        let error = error.clone();
        let pk = props.publishable_key.clone();
        let cs = props.client_secret.clone();
        let pe_opts = props.payment_element_options.clone();
        use_effect_with(stripe_ready, move |ready| {
            if *ready {
                let state = state.clone();
                let error = error.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let opts = ElementsOptions {
                        client_secret: cs.clone(),
                        appearance: None,
                    };
                    match mount_payment_element(&pk, opts, "#stripe-payment-element", pe_opts).await {
                        Ok((stripe, elements, payment_element)) => state.set(Some((stripe, elements, payment_element))),
                        Err(e) => error.set(Some(e.message)),
                    }
                });
            }
            || ()
        });
    }

    let on_click = {
        let state = state.clone();
        let loading = loading.clone();
        let error = error.clone();
        let on_success = props.on_success.clone();
        let on_error = props.on_error.clone();
        let cs = props.client_secret.clone();

        Callback::from(move |_:MouseEvent| {
            let cs = cs.clone();
            if *loading {
                return;
            }
            if let Some((stripe, elements, _pe)) = &*state {
                let stripe = stripe.clone();
                let elements = elements.clone();
                let loading = loading.clone();
                let error = error.clone();
                let on_success = on_success.clone();
                let on_error = on_error.clone();
                loading.set(true);
                error.set(None);

                wasm_bindgen_futures::spawn_local(async move {

                    // 1) Validate & collect all card/payment details
                    if let Err(err) = validate_payment_element(&elements).await {
                        on_error.emit(err.clone());
                        error.set(Some(err.message));
                        loading.set(false);
                        return;
                    }

                    // 2) Proceed with confirmPayment now that elements.submit() has run
                    let params = ConfirmPaymentParams::default();
                    match confirm_payment(&stripe, &elements, params, Some(cs.clone()), true).await {
                        PaymentResult::Success(_) => {
                            // After confirm, retrieve the PaymentIntent details to inspect status and fields
                            let stripe_js: JsValue = stripe.clone().into();
                            let retrieve_fn = js_sys::Reflect::get(&stripe_js, &JsValue::from_str("retrievePaymentIntent"))
                                .expect("retrievePaymentIntent not found")
                                .unchecked_into::<js_sys::Function>();
                            let promise = retrieve_fn
                                .call1(&stripe_js, &JsValue::from_str(&cs))
                                .expect("failed to call retrievePaymentIntent")
                                .unchecked_into::<js_sys::Promise>();
                            match wasm_bindgen_futures::JsFuture::from(promise).await {
                                Ok(result) => {
                                    let pi_js = js_sys::Reflect::get(&result, &JsValue::from_str("paymentIntent"))
                                        .expect("no paymentIntent");
                                    let pi_json: serde_json::Value = wasm_bindgen::JsValue::from(pi_js).into_serde().unwrap_or_default();
                                    let status = pi_json.get("status").and_then(|v| v.as_str()).unwrap_or_default();
                    
                                    if status == "succeeded" {
                                        // Parse result values safely
                                        let amount_cents = pi_json.get("amount_received").and_then(|v| v.as_i64())
                                            .or_else(|| pi_json.get("amount").and_then(|v| v.as_i64()))
                                            .unwrap_or(0);
                                        let amount = amount_cents as f64 / 100.0;
                                        let (last4, brand, receipt_url) = {
                                            let charges = pi_json.get("charges")
                                                .and_then(|c| c.get("data"))
                                                .and_then(|d| d.as_array());
                                            let first = charges.and_then(|arr| arr.get(0));
                                            let card = first
                                                .and_then(|f| f.get("payment_method_details"))
                                                .and_then(|pmd| pmd.get("card"));
                                            let last4 = card
                                                .and_then(|c| c.get("last4"))
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            let brand = card
                                                .and_then(|c| c.get("brand"))
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            let receipt_url = first
                                                .and_then(|f| f.get("receipt_url"))
                                                .and_then(|v| v.as_str())
                                                .map(|s| s.to_string());
                                            (last4, brand, receipt_url)
                                        };
                                        let pi_id = pi_json.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                                        on_success.emit(StripeCheckoutSuccess {
                                            amount,
                                            last4,
                                            brand,
                                            receipt_url,
                                            payment_intent_id: pi_id,
                                        });
                                    } else {
                                        // Error, not succeeded
                                        let last_payment_error = pi_json.get("last_payment_error");
                                        let msg = last_payment_error
                                            .and_then(|err| err.get("message"))
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| format!("Payment failed (status: {}). Please try another card.", status));
                                        let error_type = last_payment_error
                                            .and_then(|err| err.get("type"))
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        let code = last_payment_error
                                            .and_then(|err| err.get("code"))
                                            .and_then(|v| v.as_str())
                                            .map(|s| s.to_string());
                                        on_error.emit(StripeError { message: msg.clone(), error_type, code });
                                        error.set(Some(msg));
                                    }
                                }
                                Err(e) => {
                                    let msg = format!("Stripe API error: {:?}", e);
                                    on_error.emit(StripeError { message: msg.clone(), error_type: Some("api_error".into()), code: None });
error.set(Some(msg));
                                }
                            }
                        }
                        PaymentResult::Error(e) => {
                            on_error.emit(e.clone());
                            error.set(Some(e.message));
                        }
                    }
                    


                    loading.set(false);
                });
            }
        })
    };

    html! {
        <div class="flex flex-col gap-4 items-center w-full">
            { for props.children.iter() }
            <div id="stripe-payment-element" class="w-full mb-2" />
            <button
                type="button"
                onclick={on_click}
                disabled={!stripe_ready || *loading}
                class="rounded bg-blue-600 text-white font-semibold px-5 py-2 shadow hover:bg-blue-700 transition disabled:opacity-50">
                {
                    if *loading {
                        "Processing…".to_string()
                    } else {
                        props.button_label.clone().unwrap_or_else(|| "Pay Now".to_string())
                    }
                }
            </button>
            {
                if let Some(msg) = &*error {
                    html!{ <div class="text-red-500 text-sm">{ msg }</div> }
                } else {
                    Html::default()
                }
            }
        </div>
    }
}
