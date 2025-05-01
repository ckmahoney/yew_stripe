//! yew_stripe/src/stripe_bindings.rs
//!
//! Low‐level wasm-bindgen bindings to Stripe.js v3 for Yew.
//!
//! Exposes the raw Stripe.js handles (`JsStripe`, `JsElements`, `JsPaymentElement`)
//! and their async methods via `js_sys::Promise`.  
//! Higher-level wrappers live in `client.rs`.

use wasm_bindgen::prelude::*;
use web_sys::js_sys::Promise;

#[wasm_bindgen]
extern "C" {
    //------------------------------------------------------------------------------
    // Core Types
    //------------------------------------------------------------------------------

    /// Raw Stripe.js client handle.
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    #[derive(Debug, Clone)]
    pub type JsStripe;

    /// Raw Elements factory handle.
    #[wasm_bindgen(js_name = Elements)]
    #[derive(Debug, Clone)]
    pub type JsElements;

    /// Raw PaymentElement UI component handle.
    #[wasm_bindgen(js_name = PaymentElement)]
    #[derive(Debug, Clone)]
    pub type JsPaymentElement;

    //------------------------------------------------------------------------------
    // Constructors
    //------------------------------------------------------------------------------

    /// Construct a new `JsStripe` from your publishable key.
    ///
    /// ```js
    ///   const stripe = Stripe("pk_test_...");
    /// ```
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    pub fn new_stripe(publishable_key: &str) -> JsStripe;

    //------------------------------------------------------------------------------
    // Instance Methods
    //------------------------------------------------------------------------------

    /// `stripe.elements({ clientSecret, appearance })` → `JsElements`
    #[wasm_bindgen(method, catch, js_name = elements)]
    pub fn elements(this: &JsStripe, options: JsValue) -> Result<JsElements, JsValue>;

    /// `elements.create("payment", options)` → `JsPaymentElement`
    #[wasm_bindgen(method, catch, js_name = create)]
    pub fn create_element(
        this: &JsElements,
        element_type: &str,
        options: JsValue,
    ) -> Result<JsPaymentElement, JsValue>;

    /// `paymentElement.mount(selector)` → `()`
    #[wasm_bindgen(method, catch, js_name = mount)]
    pub fn mount(this: &JsPaymentElement, selector: &str) -> Result<(), JsValue>;

    /// `paymentElement.unmount()` → `()`
    #[wasm_bindgen(method, catch, js_name = unmount)]
    pub fn unmount(this: &JsPaymentElement) -> Result<(), JsValue>;

    /// `elements.submit()` → JS `Promise`, for field validation
    #[wasm_bindgen(method, catch, js_name = submit)]
    pub fn submit(this: &JsElements) -> Result<Promise, JsValue>;

    /// `stripe.handleCardAction(clientSecret)` → JS `Promise`
    #[wasm_bindgen(method, catch, js_name = handleCardAction)]
    pub fn handle_card_action(this: &JsStripe, client_secret: &str) -> Result<Promise, JsValue>;

    /// `stripe.confirmPayment(opts)` → JS `Promise`
    #[wasm_bindgen(method, catch, js_name = confirmPayment)]
    pub fn confirm_payment(this: &JsStripe, options: JsValue) -> Result<Promise, JsValue>;
}
