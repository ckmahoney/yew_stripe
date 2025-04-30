//! yew_stripe/src/client.rs
//!
//! High-level Rust API for integrating Stripe.js Payment Element in Yew applications.
//!
//! This module provides:
//! - `ElementsOptions` to configure Stripe Elements with a PaymentIntent client secret.
//! - `PaymentElementOptions` to customize layout and fields of the Payment Element.
//! - `ConfirmPaymentParams` for passing parameters to `stripe.confirmPayment`, such as return URLs and save-card.
//! - `mount_payment_element()` to asynchronously initialize Stripe, create Elements, and mount the Payment Element.
//! - `validate_payment_element()` to optionally validate form data before creating a PaymentIntent.
//! - `confirm_payment()` to complete the payment flow with built-in SCA/3DS support.
//! - `unmount_payment_element()` to tear down a mounted Payment Element for re-use.
//!
//! # Cargo.toml
//! ```toml
//! [dependencies]
//! wasm-bindgen = "0.2"
//! wasm-bindgen-futures = "0.4"
//! js-sys = "0.3"
//! serde = { version = "1.0", features = ["derive"] }
//! serde-wasm-bindgen = "0.5"
//! serde_json = "1.0"
//! ```
//!
//! # Example Usage
//! ```rust,ignore
//! use yew::prelude::*;
//! use crate::stripe::{
//!     ElementsOptions, ConfirmPaymentParams,
//!     mount_payment_element, confirm_payment, unmount_payment_element, PaymentResult
//! };
//! use crate::interop::use_stripejs;
//!
//! #[function_component(CheckoutForm)]
//! fn checkout_form() -> Html {
//!     let stripe_ready = use_stripejs();
//!     let stripe_state = use_state(|| None::<(JsStripe, JsElements, JsPaymentElement)>);
//!
//!     // Mount on load
//!     {
//!         let stripe_state = stripe_state.clone();
//!         use_effect_with_deps(move |ready| {
//!             if **ready {
//!                 wasm_bindgen_futures::spawn_local(async move {
//!                     let opts = ElementsOptions { client_secret: cs.clone(), appearance: None };
//!                     match mount_payment_element(&pk, opts, "#payment-element", None).await {
//!                         Ok((s, e, pe)) => stripe_state.set(Some((s, e, pe))),
//!                         Err(err)       => log::error!("Init failed: {}", err.message),
//!                     }
//!                 });
//!             }
//!             || ()
//!         }, stripe_ready);
//!     }
//!
//!     // On submit
//!     let onsubmit = {
//!         let stripe_state = stripe_state.clone();
//!         Callback::from(move |e: FocusEvent| {
//!             e.prevent_default();
//!             if let Some((s, e, pe)) = &*stripe_state {
//!                 let s = s.clone();
//!                 let e = e.clone();
//!                 let pe = pe.clone();
//!                 wasm_bindgen_futures::spawn_local(async move {
//!                     // Tear down after a previous payment (if needed)
//!                     unmount_payment_element(&pe);
//!
//!                     // Confirm new payment with save_card = true
//!                     let params = ConfirmPaymentParams {
//!                         return_url: Some("https://…".into()),
//!                         save_payment_method: Some(true),
//!                         extra: None,
//!                     };
//!                     match confirm_payment(&s, &e, params, None, true).await {
//!                         PaymentResult::Success(info) => log::info!("Paid: {:?}", info),
//!                         PaymentResult::Error(err)    => log::error!("Error: {}", err.message),
//!                     }
//!                 });
//!             }
//!         })
//!     };
//!
//!     html! {
//!         <form {onsubmit}>
//!             <div id="payment-element"></div>
//!             <button disabled={!stripe_ready}>{ "Pay Now" }</button>
//!         </form>
//!     }
//! }
//! ```


use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{Object, Reflect};
use serde::{Serialize, Deserialize};
use serde_wasm_bindgen::{to_value, from_value};
use crate::bindings::{
    new_stripe,
    Stripe as JsStripe,
    Elements as JsElements,
    PaymentElement as JsPaymentElement,
};

/// Configuration for `stripe.elements({ clientSecret, appearance })`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElementsOptions {
    /// The PaymentIntent client secret returned by your backend.
    #[serde(rename = "clientSecret")]
    pub client_secret: String,

    /// Optional Stripe Elements appearance settings.
    #[serde(rename = "appearance", skip_serializing_if = "Option::is_none")]
    pub appearance: Option<serde_json::Value>,
}

/// Optional layout/customization for the mounted Payment Element.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaymentElementOptions {
    /// Layout mode: `"tabs"` or `"accordion"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,

    /// Any other JSON-serializable settings (e.g., fields).
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

/// Parameters for `stripe.confirmPayment({ confirmParams, ... })`.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfirmPaymentParams {
    /// For redirect-based flows: where to send the customer on success.
    #[serde(rename = "return_url", skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,

    /// Whether to save the payment method for off-session use.
    #[serde(rename = "save_payment_method", skip_serializing_if = "Option::is_none")]
    pub save_payment_method: Option<bool>,

    /// Any additional confirm params (e.g. shipping info).
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

/// Minimal representation of a confirmed PaymentIntent.
#[derive(Clone, Debug)]
pub struct PaymentIntentInfo {
    /// Stripe’s internal identifier, e.g. `pi_1Fxxxxxx`.
    pub id: String,
    /// Final status, e.g. `"succeeded"`.
    pub status: String,
}

/// Strongly-typed outcome of attempting to confirm a payment.
#[derive(Debug)]
pub enum PaymentResult {
    /// The PaymentIntent succeeded. Contains basic info.
    Success(PaymentIntentInfo),
    /// Something went wrong. Contains Stripe’s error details.
    Error(StripeError),
}

/// Representation of a Stripe.js error object.
#[derive(Clone, Debug, Deserialize)]
pub struct StripeError {
    /// Human-readable message.
    pub message: String,
    /// Stripe’s error type, e.g. `"card_error"`.
    #[serde(rename = "type", default)]
    pub error_type: Option<String>,
    /// Optional Stripe error code, e.g. `"card_declined"`.
    #[serde(default)]
    pub code: Option<String>,
}

/// Initialize Stripe.js, create an Elements instance, and mount a PaymentElement.
///
/// # Arguments
///
/// * `publishable_key` – Your Stripe publishable key (starts with `pk_`).
/// * `elements_options` – Must include `client_secret`.
/// * `mount_id` – CSS selector or DOM id, e.g. `"#payment-element"`.
/// * `pe_options` – Optional layout/customization.
///
/// # Returns
///
/// On success, returns `(JsStripe, JsElements, JsPaymentElement)`. On failure,
/// returns a `StripeError`.
///
pub async fn mount_payment_element(
    publishable_key: &str,
    elements_options: ElementsOptions,
    mount_id: &str,
    pe_options: Option<PaymentElementOptions>,
) -> Result<(JsStripe, JsElements, JsPaymentElement), StripeError> {
    // 1) Create Stripe instance
    let stripe = new_stripe(publishable_key);

    // 2) Build JS args for elements()
    let opts_js = to_value(&elements_options)
        .map_err(serde_error_to_stripe_error)?;
    let elements = stripe
        .elements(opts_js)
        .map_err(js_to_stripe_error)?;

    // 3) Build JS args for create("payment", ...)
    let pe_opts_js = pe_options
        .map(|opts| to_value(&opts).expect("PaymentElementOptions serialization failed"))
        .unwrap_or(JsValue::undefined());
    let payment_element = elements
        .create_element("payment", pe_opts_js)
        .map_err(js_to_stripe_error)?;

    // 4) Mount into DOM
    payment_element
        .mount(mount_id)
        .map_err(js_to_stripe_error)?;

    Ok((stripe, elements, payment_element))
}

/// Optionally validate collected form data before creating a PaymentIntent.
/// Only needed if Elements was initialized **without** a `clientSecret`.
///
/// # Errors
///
/// Returns `Err(StripeError)` if validation fails or JS throws.
///
pub async fn validate_payment_element(
    elements: &JsElements,
) -> Result<(), StripeError> {
    let promise = elements
        .submit()
        .map_err(js_to_stripe_error)?;
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(js_to_stripe_error)
}

/// Confirm a PaymentIntent using the mounted Payment Element, handling SCA/3DS automatically.
///
/// # Arguments
///
/// * `stripe` – The `JsStripe` from `mount_payment_element`.
/// * `elements` – The `JsElements` from `mount_payment_element`.
/// * `params` – Your `ConfirmPaymentParams`.
/// * `client_secret` – `Some(...)` for two-step flows, or `None` if you passed `clientSecret` earlier.
/// * `redirect_if_required` – `true` to use `"if_required"` (recommended).
///
pub async fn confirm_payment(
    stripe: &JsStripe,
    elements: &JsElements,
    params: ConfirmPaymentParams,
    client_secret: Option<String>,
    redirect_if_required: bool,
) -> PaymentResult {
    // Build the JS options object dynamically
    let opts = Object::new();
    if let Some(cs) = client_secret {
        Reflect::set(&opts, &JsValue::from_str("paymentElement"), elements.as_ref()).unwrap();
        Reflect::set(&opts, &JsValue::from_str("clientSecret"), &JsValue::from_str(&cs)).unwrap();
    } else {
        Reflect::set(&opts, &JsValue::from_str("elements"), elements.as_ref()).unwrap();
    }
    let params_js = to_value(&params).expect("ConfirmPaymentParams serialization failed");
    Reflect::set(&opts, &JsValue::from_str("confirmParams"), &params_js).unwrap();
    if redirect_if_required {
        Reflect::set(&opts, &JsValue::from_str("redirect"), &JsValue::from_str("if_required")).unwrap();
    }

    // Call stripe.confirmPayment(...)
    let promise = match stripe.confirm_payment(opts.into()) {
        Ok(p) => p,
        Err(e) => return PaymentResult::Error(js_to_stripe_error(e)),
    };

    // Await the JS Promise
    match JsFuture::from(promise).await {
        Ok(js_val) => {
            // Try to deserialize into StripeError first
            if let Ok(err) = from_value::<StripeError>(js_val.clone()) {
                return PaymentResult::Error(err);
            }
            // Otherwise extract PaymentIntent info
            let intent = Reflect::get(&js_val, &JsValue::from_str("paymentIntent"))
                .ok()
                .and_then(|pi| Reflect::get(&pi, &JsValue::from_str("id")).ok())
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let status = Reflect::get(&js_val, &JsValue::from_str("status"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "succeeded".into());
            PaymentResult::Success(PaymentIntentInfo { id: intent, status })
        }
        Err(js_err) => PaymentResult::Error(js_to_stripe_error(js_err)),
    }
}

/// Tear down a mounted PaymentElement so it can be re-mounted for another payment.
///
/// # Errors
///
/// Returns `Err(StripeError)` if unmount fails.
///
pub fn unmount_payment_element(
    payment_element: &JsPaymentElement
) -> Result<(), StripeError> {
    payment_element.unmount().map_err(js_to_stripe_error)
}

/// Manually trigger off-session 3DS/SCA challenges.
///
/// # Arguments
///
/// * `stripe` – Your `JsStripe` instance.
/// * `client_secret` – The PaymentIntent client secret for off-session flows.
///
/// # Errors
///
/// Returns `Err(StripeError)` if Stripe.js rejects.
///
pub async fn handle_card_action(
    stripe: &JsStripe,
    client_secret: &str
) -> Result<(), StripeError> {
    let promise = stripe
        .handle_card_action(client_secret)
        .map_err(js_to_stripe_error)?;
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(js_to_stripe_error)
}

/// Convert any caught `JsValue` into a `StripeError` with best effort.
fn js_to_stripe_error(value: JsValue) -> StripeError {
    from_value::<StripeError>(value.clone()).unwrap_or_else(|_| StripeError {
        message: value.as_string().unwrap_or_else(|| format!("{:?}", value)),
        error_type: None,
        code: None,
    })
}

/// Convert a `serde_wasm_bindgen::Error` (from `to_value`) into `StripeError`.
fn serde_error_to_stripe_error(err: serde_wasm_bindgen::Error) -> StripeError {
    StripeError {
        message: err.to_string(),
        error_type: None,
        code: None,
    }
}