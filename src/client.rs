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

use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::{Object, Reflect, Function};
use serde_wasm_bindgen::{to_value, from_value};
use serde::{Serialize, Deserialize};
use serde_json::Value as JsonValue;
use crate::bindings::{
    new_stripe, Stripe as JsStripe, Elements as JsElements, PaymentElement as JsPaymentElement,
};

/// Configuration for `stripe.elements(...)`.
#[derive(Serialize, Debug)]
pub struct ElementsOptions {
    /// The client secret from your backend PaymentIntent.
    #[serde(rename = "clientSecret")]
    pub client_secret: String,

    /// Optional Stripe Elements appearance settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<JsonValue>,
}

/// Customization for `elements.create("payment", ...)`.
#[derive(Serialize, Debug)]
pub struct PaymentElementOptions {
    /// Layout style: `"tabs"` or `"accordion"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,

    /// Additional JSON-serializable settings (e.g. fields).
    #[serde(flatten)]
    pub extra: Option<JsonValue>,
}

/// Parameters passed into `stripe.confirmPayment({ confirmParams, ... })`.
#[derive(Serialize, Debug)]
pub struct ConfirmPaymentParams {
    /// Where to redirect on success (only for redirect-based flows).
    #[serde(rename = "return_url", skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,

    /// Whether to save the payment method for future off-session use.
    #[serde(rename = "save_payment_method", skip_serializing_if = "Option::is_none")]
    pub save_payment_method: Option<bool>,

    /// Additional confirm params (e.g. shipping, payment_method_data).
    #[serde(flatten)]
    pub extra: Option<JsonValue>,
}

/// Outcome of attempting to confirm a payment.
#[derive(Debug)]
pub enum PaymentResult {
    /// PaymentIntent succeeded; contains minimal details.
    Success(PaymentIntentInfo),
    /// An error occurred; contains Stripe’s error message and code.
    Error(StripeError),
}

/// Minimal information about a confirmed PaymentIntent.
#[derive(Debug)]
pub struct PaymentIntentInfo {
    /// The PaymentIntent’s identifier (e.g. `pi_12345`).
    pub id: String,
    /// The final status (e.g. `"succeeded"`).
    pub status: String,
}

/// Structured representation of a Stripe.js error.
#[derive(Debug, Deserialize)]
pub struct StripeError {
    /// Human-readable message explaining what went wrong.
    pub message: String,
    /// Stripe’s error type (e.g. `"card_error"`).
    #[serde(rename = "type", default)]
    pub error_type: Option<String>,
    /// Optional Stripe error code (e.g. `"card_declined"`).
    #[serde(default)]
    pub code: Option<String>,
}

/// Initialize Stripe.js, create Elements, and mount the Payment Element.
///
/// # Arguments
/// - `publishable_key`: Your Stripe publishable key (e.g. `"pk_test_…"`)
/// - `elements_options`: Configuration including `client_secret`.
/// - `mount_id`: CSS selector or ID (e.g. `"#payment-element"`).
/// - `pe_options`: Optional `PaymentElementOptions` for layout/customization.
///
/// # Returns
/// `(Stripe, Elements, PaymentElement)` on success, or a `StripeError`.
pub async fn mount_payment_element(
    publishable_key: &str,
    elements_options: ElementsOptions,
    mount_id: &str,
    pe_options: Option<PaymentElementOptions>,
) -> Result<(JsStripe, JsElements, JsPaymentElement), StripeError> {
    let stripe = new_stripe(publishable_key);

    let opts_js = to_value(&elements_options)
        .map_err(|e| StripeError { message: e.to_string(), error_type: None, code: None })?;
    let elements = stripe
        .elements(opts_js)
        .map_err(js_error_to_stripe_error)?;

    let pe_js = pe_options
        .map(|opt| to_value(&opt).expect("PaymentElementOptions serialization failed"))
        .unwrap_or(JsValue::undefined());
    let payment_el = elements
        .create_element("payment", pe_js)
        .map_err(js_error_to_stripe_error)?;
    payment_el
        .mount(mount_id)
        .map_err(js_error_to_stripe_error)?;

    Ok((stripe, elements, payment_el))
}

/// Optionally validate the Payment Element form before creating an intent.
/// Required only if Elements was initialized without a client secret.
pub async fn validate_payment_element(
    elements: &JsElements,
) -> Result<(), StripeError> {
    let promise = elements
        .submit()
        .map_err(js_error_to_stripe_error)?;
    JsFuture::from(promise)
        .await
        .map(|_| ())
        .map_err(js_error_to_stripe_error)
}

/// Confirm a PaymentIntent using the mounted Payment Element.
///
/// # Arguments
/// - `stripe`: Stripe instance from `mount_payment_element`.
/// - `elements`: Elements instance from `mount_payment_element`.
/// - `params`: `ConfirmPaymentParams` (e.g. `return_url`, `save_payment_method`).
/// - `client_secret`: `Some(...)` for two‐step flow, `None` for one‐step.
/// - `redirect_if_required`: `true` for `"if_required"` behavior.
///
/// # Returns
/// `PaymentResult::Success` or `PaymentResult::Error`.
pub async fn confirm_payment(
    stripe: &JsStripe,
    elements: &JsElements,
    params: ConfirmPaymentParams,
    client_secret: Option<String>,
    redirect_if_required: bool,
) -> PaymentResult {
    let opts = Object::new();
    if let Some(cs) = client_secret {
        Reflect::set(&opts, &"paymentElement".into(), elements.as_ref()).unwrap();
        Reflect::set(&opts, &"clientSecret".into(), &cs.into()).unwrap();
    } else {
        Reflect::set(&opts, &"elements".into(), elements.as_ref()).unwrap();
    }
    let cp_js = to_value(&params).expect("ConfirmPaymentParams serialization failed");
    Reflect::set(&opts, &"confirmParams".into(), &cp_js).unwrap();
    if redirect_if_required {
        Reflect::set(&opts, &"redirect".into(), &"if_required".into()).unwrap();
    }

    let promise = match stripe.confirm_payment(opts.into()) {
        Ok(p) => p,
        Err(e) => return PaymentResult::Error(js_error_to_stripe_error(e)),
    };

    match JsFuture::from(promise).await {
        Ok(js_val) => {
            if let Ok(err) = from_value::<StripeError>(js_val.clone()) {
                return PaymentResult::Error(err);
            }
            let pi = Reflect::get(&js_val, &"paymentIntent".into())
                .ok()
                .and_then(|v| Reflect::get(&v, &"id".into()).ok())
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let st = Reflect::get(&js_val, &"status".into())
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "succeeded".into());
            PaymentResult::Success(PaymentIntentInfo { id: pi, status: st })
        }
        Err(js_err) => PaymentResult::Error(js_error_to_stripe_error(js_err)),
    }
}

/// Tear down a mounted Payment Element so it can be re-mounted for a new payment.
pub fn unmount_payment_element(pe: &JsPaymentElement) {
    pe.unmount().map_err(js_error_to_stripe_error)?;
}

/// Convert any JS exception or Promise rejection into `StripeError`.
fn js_error_to_stripe_error(js_val: JsValue) -> StripeError {
    if let Ok(err) = from_value::<StripeError>(js_val.clone()) {
        err
    } else {
        StripeError {
            message: js_val.as_string().unwrap_or_else(|| format!("{:?}", js_val)),
            error_type: None,
            code: None,
        }
    }
}
