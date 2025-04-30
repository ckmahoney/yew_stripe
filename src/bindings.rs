//! yew_stripe/src/bindings.rs
//!
//! Low‐level wasm‐bindgen bindings to Stripe.js v3 for use in Yew applications.
//!
//! This module exposes the core Stripe.js objects and methods needed to build
//! a Payment Element integration entirely in Rust, without inline JavaScript.
//!
//! # Overview
//! - `Stripe` ‒ primary client object (created via `Stripe(publishableKey)`).
//! - `Elements` ‒ factory for prebuilt UI components.
//! - `PaymentElement` ‒ the single UI component for collecting payment details.
//! - `Promise` ‒ JavaScript Promise, returned by async Stripe methods.
//!
//! # Key Bindings
//! 1. **Global `Stripe` constructor**  
//!    Creates a new client: `let stripe = Stripe("pk_test_...");`  
//! 2. **`stripe.elements(options)`**  
//!    Initializes Elements with `clientSecret` and appearance settings.  
//! 3. **`elements.create("payment", options)`**  
//!    Builds a `PaymentElement` for your form.  
//! 4. **`paymentElement.mount(selector)`**  
//!    Attaches the element to the DOM.  
//! 5. **`elements.submit()`** *(optional)*  
//!    Validates collected data before intent creation.  
//! 6. **`stripe.confirmPayment(opts)`**  
//!    Confirms PaymentIntent, handling SCA/3DS flows automatically.  
//! 7. **`paymentElement.unmount()`**  
//!    Tears down the mounted element so it can be re-mounted later.  
//! 8. **`stripe.handleCardAction(clientSecret)`** *(optional)*  
//!    Manually trigger 3DS challenge handling for off-session flows.
//!
//! # Cargo.toml
//! ```toml
//! [dependencies]
//! wasm-bindgen = "0.2"
//! js-sys = "0.3"
//! serde = { version = "1.0", features = ["derive"] }
//! serde-wasm-bindgen = "0.5"
//! ```
//!
//! # Example
//! ```ignore
//! use wasm_bindgen::JsValue;
//! use crate::bindings::{Stripe, Elements, PaymentElement};
//!
//! // Load Stripe.js separately via `yew-interop`.
//! let stripe: Stripe = Stripe::new("pk_test_...");
//! let opts = JsValue::from_serde(&serde_json::json!({ "clientSecret": cs })).unwrap();
//! let elements: Elements = stripe.elements(opts).unwrap();
//! let pe_opts = JsValue::undefined();
//! let payment_element: PaymentElement = elements.create("payment", pe_opts).unwrap();
//! payment_element.mount("#payment-element").unwrap();
//! // Later, to tear down:
//! payment_element.unmount().unwrap();
//! // For off-session 3DS handling:
//! stripe.handle_card_action("pi_client_secret_...").unwrap();
//! ```
//!
//! See higher-level wrappers in `client.rs` for async/await and error handling.

use wasm_bindgen::prelude::*;
use web_sys::js_sys::Promise;

#[wasm_bindgen]
extern "C" {
    //------------------------------------------------------------------------------
    // Types
    //------------------------------------------------------------------------------

    /// The primary Stripe client. Created by calling the global `Stripe(publishableKey)`.
    #[wasm_bindgen(js_name = Stripe)]
    #[derive(Debug, Clone)]
    pub type Stripe;

    /// The Elements factory, used to create UI components (e.g., PaymentElement).
    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub type Elements;

    /// A prebuilt UI component for collecting payment details (card, UPI, wallets).
    #[wasm_bindgen]
    #[derive(Debug)]
    pub type PaymentElement;

    //------------------------------------------------------------------------------
    // Functions & Methods
    //------------------------------------------------------------------------------

    /// Global constructor: `Stripe(publishableKey)` → `Stripe` instance.
    ///
    /// # Arguments
    /// * `publishable_key` – Your Stripe publishable API key (e.g. `"pk_test_…"`).  
    /// # Errors
    /// Throws if Stripe.js isn't loaded or key is invalid.
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    pub fn new_stripe(publishable_key: &str) -> Stripe;

    /// Initializes an `Elements` instance linked to a PaymentIntent.
    ///
    /// # Arguments
    /// * `options` – JS object, must include `clientSecret` (and optional `appearance`).
    /// # Errors
    /// Throws on invalid or missing `clientSecret`.
    #[wasm_bindgen(method, catch, js_name = elements)]
    pub fn elements(this: &Stripe, options: JsValue) -> Result<Elements, JsValue>;

    /// Creates a UI component on an `Elements` instance.
    ///
    /// # Arguments
    /// * `element_type` – e.g. `"payment"`, `"card"`.  
    /// * `options` – JS object with component-specific settings (layout, fields).  
    /// # Errors
    /// Throws on unsupported element type or bad options.
    #[wasm_bindgen(method, catch, js_name = create)]
    pub fn create_element(
        this: &Elements,
        element_type: &str,
        options: JsValue,
    ) -> Result<PaymentElement, JsValue>;

    /// Mounts a `PaymentElement` into the page.
    ///
    /// # Arguments
    /// * `selector` – CSS selector or element ID (e.g. `"#payment-element"`).  
    /// # Errors
    /// Throws if the selector is not found or element already mounted.
    #[wasm_bindgen(method, catch)]
    pub fn mount(this: &PaymentElement, selector: &str) -> Result<(), JsValue>;

    /// Tears down a mounted `PaymentElement`.
    ///
    /// # Errors
    /// Throws if the element is not mounted or on unmount failure.
    #[wasm_bindgen(method, catch)]
    pub fn unmount(this: &PaymentElement) -> Result<(), JsValue>;

    /// Validates all fields in an `Elements` form.
    ///
    /// Only used in two-step flows where you collect data
    /// before creating a PaymentIntent.
    ///
    /// # Returns
    /// A JS `Promise` that resolves on success or rejects with an error.
    #[wasm_bindgen(method, catch)]
    pub fn submit(this: &Elements) -> Result<Promise, JsValue>;

    /// Manually handle 3DS/SCA challenge for off-session PaymentIntents.
    ///
    /// # Arguments
    /// * `client_secret` – The PaymentIntent client secret.
    /// # Returns
    /// A JS `Promise` that resolves when the challenge completes or rejects on error.
    #[wasm_bindgen(method, catch, js_name = handleCardAction)]
    pub fn handle_card_action(this: &Stripe, client_secret: &str) -> Result<Promise, JsValue>;

    /// Confirms a PaymentIntent using a `PaymentElement`.
    ///
    /// # Arguments
    /// * `options` – JS object.  
    ///    - Either `{ elements, confirmParams, redirect }`  
    ///    - Or `{ paymentElement, clientSecret, confirmParams, redirect }` for two-step flows.  
    ///
    /// * `confirmParams` keys: e.g. `return_url`.  
    /// * `redirect`: `"if_required"` vs `"always"`.  
    ///
    /// # Returns
    /// A JS `Promise` resolving to either a success result or an error object.
    #[wasm_bindgen(method, catch, js_name = confirmPayment)]
    pub fn confirm_payment(this: &Stripe, options: JsValue) -> Result<Promise, JsValue>;
}
