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
    // Core Types
    //------------------------------------------------------------------------------

    /// The primary Stripe client. Construct with `new_stripe(publishable_key)`.
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    #[derive(Debug, Clone)]
    pub type Stripe;

    /// Factory for creating UI components (PaymentElement, CardElement, etc.).
    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub type Elements;

    /// Prebuilt UI component for collecting payment details.
    #[wasm_bindgen]
    #[derive(Debug)]
    pub type PaymentElement;

    //------------------------------------------------------------------------------
    // Constructors & Methods
    //------------------------------------------------------------------------------

    /// Create a new `Stripe` instance:
    /// ```js
    ///   const stripe = Stripe("pk_test_...");
    /// ```
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    pub fn new_stripe(publishable_key: &str) -> Stripe;

    /// Initialize Elements linked to a PaymentIntent:
    /// ```js
    ///   const elements = stripe.elements({ clientSecret: "...", appearance: {...} });
    /// ```
    #[wasm_bindgen(method, catch, js_name = elements)]
    pub fn elements(this: &Stripe, options: JsValue) -> Result<Elements, JsValue>;

    /// Create a UI component on an `Elements` instance:
    /// ```js
    ///   const pe = elements.create("payment", { layout: "tabs" });
    /// ```
    #[wasm_bindgen(method, catch, js_name = create)]
    pub fn create_element(
        this: &Elements,
        element_type: &str,
        options: JsValue,
    ) -> Result<PaymentElement, JsValue>;

    /// Mount a mounted `PaymentElement` into the DOM:
    /// ```js
    ///   pe.mount("#payment-element");
    /// ```
    #[wasm_bindgen(method, catch)]
    pub fn mount(this: &PaymentElement, selector: &str) -> Result<(), JsValue>;

    /// Unmount a `PaymentElement` so it can be re-mounted:
    #[wasm_bindgen(method, catch)]
    pub fn unmount(this: &PaymentElement) -> Result<(), JsValue>;

    /// Validate all fields in an `Elements` form:
    /// ```js
    ///   elements.submit().then(...);
    /// ```
    #[wasm_bindgen(method, catch)]
    pub fn submit(this: &Elements) -> Result<Promise, JsValue>;

    /// Manually handle off-session SCA/3DS:
    /// ```js
    ///   stripe.handleCardAction(clientSecret).then(...);
    /// ```
    #[wasm_bindgen(method, catch, js_name = handleCardAction)]
    pub fn handle_card_action(this: &Stripe, client_secret: &str) -> Result<Promise, JsValue>;

    /// Confirm a PaymentIntent using a `PaymentElement` or `elements`:
    /// ```js
    ///   stripe.confirmPayment({ elements, confirmParams, redirect: "if_required" });
    /// ```
    #[wasm_bindgen(method, catch, js_name = confirmPayment)]
    pub fn confirm_payment(this: &Stripe, options: JsValue) -> Result<Promise, JsValue>;
}
