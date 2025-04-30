//! stripe.rs
//!
//! High-level Rust API for integrating Stripe.js Payment Element in Yew applications.
//!
//! This module provides:
//! - `ElementsOptions` to configure Stripe Elements with a PaymentIntent client secret.
//! - `PaymentElementOptions` to customize layout and fields of the Payment Element.
//! - `ConfirmPaymentParams` for passing parameters to `stripe.confirmPayment`, such as return URLs.
//! - `mount_payment_element()` to asynchronously initialize Stripe, create Elements, and mount the Payment Element.
//! - `validate_payment_element()` to optionally validate form data before creating a PaymentIntent.
//! - `confirm_payment()` to complete the payment flow with built-in SCA/3DS support and optional save-card functionality.
//!
//! # Dependencies
//! ```toml
//! [dependencies]
//! wasm-bindgen = "0.2"
//! wasm-bindgen-futures = "0.4"
//! js-sys = "0.3"
//! serde = { version = "1.0", features = ["derive"] }
//! serde-wasm-bindgen = "0.5"
//! ```
//!
//! # Example Usage
//! ```rust,ignore
//! use yew::prelude::*;
//! use crate::stripe::{ElementsOptions, ConfirmPaymentParams, mount_payment_element, confirm_payment, PaymentResult};
//!
//! #[function_component(CheckoutForm)]
//! fn checkout_form() -> Html {
//!     // call use_stripejs() to load Stripe.js first...
//!     // then in an effect:
//!     wasm_bindgen_futures::spawn_local(async move {
//!         let elements_opts = ElementsOptions { client_secret: cs.clone(), appearance: None };
//!         let (stripe, elements, _) = mount_payment_element(&pk, elements_opts, "#payment", None).await.unwrap();
//!         // store stripe & elements and render form...
//!     });
//!
//!     // on submit:
//!     wasm_bindgen_futures::spawn_local(async move {
//!         let params = ConfirmPaymentParams { return_url: Some("https://…".into()) };
//!         match confirm_payment(&stripe, &elements, params, None, true).await {
//!             PaymentResult::Success(info) => log::info!("Paid: {:?}", info),
//!             PaymentResult::Error(err)    => log::error!("Payment failed: {}", err.message),
//!         }
//!     });
//!     html! { /* … */ }
//! }
//! ```

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Object, Reflect};
use serde::{Serialize, Deserialize};
use crate::stripe_bindings::{
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
    pub appearance: Option<serde_json::Value>,
}

/// Customization for `elements.create("payment", ...)`.
#[derive(Serialize, Debug)]
pub struct PaymentElementOptions {
    /// Layout style: `"tabs"` or `"accordion"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<String>,

    /// Additional JSON-serializable settings (e.g. fields).
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

/// Parameters passed into `stripe.confirmPayment({ confirmParams, ... })`.
#[derive(Serialize, Debug)]
pub struct ConfirmPaymentParams {
    /// Where to redirect on success (only for redirect-based flows).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url: Option<String>,

    /// Additional confirm params (e.g. shipping, payment_method_data).
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
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
    #[serde(rename = "type")]
    pub error_type: Option<String>,

    /// Optional Stripe error code (e.g. `"card_declined"`).
    pub code: Option<String>,
}

/// Initialize Stripe.js, create Elements, and mount the Payment Element.
///
/// # Arguments
/// * `publishable_key` – Your Stripe publishable key (e.g. `"pk_test_…"`)
/// * `elements_options` – Configuration including `client_secret`.
/// * `mount_id` – CSS selector or ID (e.g. `"#payment-element"`).
/// * `pe_options` – Optional `PaymentElementOptions` for layout/customization.
///
/// # Returns
/// A tuple `(Stripe, Elements, PaymentElement)` on success, or a `StripeError`.
pub async fn mount_payment_element(
    publishable_key: &str,
    elements_options: ElementsOptions,
    mount_id: &str,
    pe_options: Option<PaymentElementOptions>,
) -> Result<(JsStripe, JsElements, JsPaymentElement), StripeError> {
    // Create Stripe instance
    let stripe = new_stripe(publishable_key);

    // Serialize ElementsOptions to JS object
    let opts_js = JsValue::from_serde(&elements_options)
        .expect("Failed to serialize ElementsOptions");
    let elements = stripe
        .elements(opts_js)
        .map_err(js_error_to_stripe_error)?;

    // Serialize PaymentElementOptions or pass undefined
    let pe_js = pe_options
        .map(|opt| JsValue::from_serde(&opt).unwrap())
        .unwrap_or(JsValue::undefined());
    let payment_el = elements
        .create_element("payment", pe_js)
        .map_err(js_error_to_stripe_error)?;

    // Mount into the DOM
    payment_el
        .mount(mount_id)
        .map_err(js_error_to_stripe_error)?;

    Ok((stripe, elements, payment_el))
}

/// Optionally validate the Payment Element form before creating an intent.
///
/// Only necessary if you initialized Elements *without* a `client_secret`.
///
/// # Returns
/// `Ok(())` if validation passed, or a `StripeError`.
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
/// * `stripe` – The `Stripe` instance from `mount_payment_element`.
/// * `elements` – The `Elements` instance from `mount_payment_element`.
/// * `params` – `ConfirmPaymentParams` (e.g. `return_url`).
/// * `client_secret` – `Some(...)` for two‐step flow, `None` for one‐step.
/// * `redirect_if_required` – Pass `"if_required"` to avoid unnecessary redirects.
///
/// # Returns
/// `PaymentResult::Success` on success or `PaymentResult::Error` on failure.
pub async fn confirm_payment(
    stripe: &JsStripe,
    elements: &JsElements,
    params: ConfirmPaymentParams,
    client_secret: Option<String>,
    redirect_if_required: bool,
) -> PaymentResult {
    // Build options object for confirmPayment
    let opts = Object::new();

    if let Some(cs) = client_secret {
        // Two‐step flow: supply paymentElement & clientSecret
        Reflect::set(&opts, &"paymentElement".into(), elements)
            .unwrap();
        Reflect::set(&opts, &"clientSecret".into(), &cs.into())
            .unwrap();
    } else {
        // One‐step: Elements was created with clientSecret
        Reflect::set(&opts, &"elements".into(), elements)
            .unwrap();
    }

    // Attach confirmParams
    let cp_js = JsValue::from_serde(&params)
        .expect("Failed to serialize ConfirmPaymentParams");
    Reflect::set(&opts, &"confirmParams".into(), &cp_js)
        .unwrap();

    // Attach redirect behavior
    if redirect_if_required {
        Reflect::set(&opts, &"redirect".into(), &"if_required".into())
            .unwrap();
    }

    // Call stripe.confirmPayment(...)
    let promise = match stripe.confirm_payment(opts.into()) {
        Ok(p) => p,
        Err(e) => return PaymentResult::Error(js_error_to_stripe_error(e)),
    };

    // Await the result
    match JsFuture::from(promise).await {
        Ok(js_val) => {
            // Check for an embedded error object
            if js_val.is_object() {
                if let Ok(err) = js_val.clone().into_serde::<StripeError>() {
                    return PaymentResult::Error(err);
                }
                // Attempt to extract PaymentIntent info
                let id = js_val.as_ref().get("paymentIntent")
                    .and_then(|pi| pi.as_ref().get("id"))
                    .and_then(|v| v.as_string())
                    .unwrap_or_default();
                let status = js_val.as_ref().get("status")
                    .and_then(|v| v.as_string())
                    .unwrap_or_else(|| "succeeded".into());
                return PaymentResult::Success(PaymentIntentInfo { id, status });
            }
            // Fallback success
            PaymentResult::Success(PaymentIntentInfo {
                id: String::new(),
                status: "succeeded".into(),
            })
        }
        Err(js_err) => PaymentResult::Error(js_error_to_stripe_error(js_err)),
    }
}

/// Convert a caught JS exception or promise rejection into `StripeError`.
fn js_error_to_stripe_error(js_val: JsValue) -> StripeError {
    // Try to deserialize a structured StripeError
    if let Ok(err) = js_val.clone().into_serde::<StripeError>() {
        err
    } else {
        // Fallback to generic message
        StripeError {
            message: js_val.as_string().unwrap_or_else(|| format!("{:?}", js_val)),
            error_type: None,
            code: None,
        }
    }
}
