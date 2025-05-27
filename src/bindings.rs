//! yew_stripe/src/stripe_bindings.rs
//!
//! Low‐level wasm-bindgen bindings to Stripe.js v3 for Yew.
//!
//! This module provides direct, one-to-one Rust bindings for the core
//! Stripe.js v3 classes and methods. Use these types only if you need
//! raw access to the underlying JS API. For most integration needs,
//! prefer the high-level, ergonomic wrappers in `client.rs`, which handle
//! JSON conversion, error mapping, SCA/3DS flows, and Yew-friendly async patterns.
//!
//! # Core Types
//!
//! - [`JsStripe`]: the primary Stripe client instance.
//! - [`JsElements`]: factory for Stripe Elements.
//! - [`JsPaymentElement`]: the Payment Element UI component.
//!
//! # Conventions
//!
//! - Methods marked `catch` return `Result<…, JsValue>` for JS exceptions.
//! - Async methods return `js_sys::Promise`, await via `wasm_bindgen_futures::JsFuture`.
//!
//! # External Docs
//!
//! - Stripe.js v3 reference: <https://stripe.com/docs/js>
//! - Payment Element guide:  <https://stripe.com/docs/js/payment_element>
//!
//! For a turnkey Rust wrapper, see [`mount_payment_element`](crate::client::mount_payment_element)
//! in `client.rs`, which handles JSON conversion, error mapping, SCA/3DS and Yew async patterns.

use wasm_bindgen::prelude::*;
use web_sys::js_sys::Promise;

#[wasm_bindgen]
extern "C" {
    //------------------------------------------------------------------------------
    // Core Types
    //------------------------------------------------------------------------------

    /// The primary Stripe.js client handle.
    ///
    /// Corresponds to the JS global `window.Stripe` constructor.
    ///
    /// Use [`new_stripe`] to instantiate.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stripe: JsStripe = new_stripe("pk_test_...");
    /// ```
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    #[derive(Debug, Clone)]
    pub type JsStripe;

    /// Factory for creating Stripe Elements (UI components).
    ///
    /// Corresponds to the JS `stripe.elements({...})` call.
    /// From an Elements instance you can create e.g. a Payment Element,
    /// Card Element, etc.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opts = js_sys::Object::from_entries(&…)?;
    /// let elements: JsElements = stripe.elements(opts.into()).unwrap();
    /// ```
    #[wasm_bindgen(js_name = Elements)]
    #[derive(Debug, Clone)]
    pub type JsElements;

    /// The Stripe.js Payment Element UI component handle.
    ///
    /// Created via `elements.create("payment", options)`,
    /// can be mounted/unmounted in the DOM.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let payment_el: JsPaymentElement = elements
    ///     .create("payment", opts_js)
    ///     .unwrap();
    /// ```
    #[wasm_bindgen(js_name = PaymentElement)]
    #[derive(Debug, Clone)]
    pub type JsPaymentElement;

    //------------------------------------------------------------------------------
    // Constructors
    //------------------------------------------------------------------------------

    /// Create a new Stripe.js client from your publishable key.
    ///
    /// Wraps the JS global `Stripe(publishableKey)` constructor.
    ///
    /// # Arguments
    ///
    /// - `publishable_key`: Your Stripe publishable key (starts with `pk_`).
    ///
    /// # Returns
    ///
    /// A [`JsStripe`] instance ready to invoke further methods.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let stripe = new_stripe("pk_live_...");
    /// ```
    #[wasm_bindgen(js_name = Stripe, js_namespace = window)]
    pub fn new_stripe(publishable_key: &str) -> JsStripe;

    //------------------------------------------------------------------------------
    // Instance Methods
    //------------------------------------------------------------------------------

    /// Initialize a Stripe Elements factory.
    ///
    /// Calls `stripe.elements(options)` in JS.  
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsStripe` instance.
    /// - `options`: a JSON object with at least:
    ///   - `clientSecret` (string): your PaymentIntent client secret.
    ///   - `appearance` (object, optional): Elements appearance config.
    ///
    /// # Returns
    ///
    /// - `Ok(JsElements)`: the Elements factory on success.
    /// - `Err(JsValue)`: a JS exception (invalid options, missing keys).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opts = serde_wasm_bindgen::to_value(&ElementsOptions { client_secret: "...".into(), appearance: None }).unwrap();
    /// let elements = stripe.elements(opts).unwrap();
    /// ```
    #[wasm_bindgen(method, catch, js_name = elements)]
    pub fn elements(this: &JsStripe, options: JsValue) -> Result<JsElements, JsValue>;

    /// Create a Stripe Element UI component.
    ///
    /// Corresponds to `elements.create(type, options)` in JS.
    ///
    /// **Supported types**: `"payment"` (tested). Stripe.js also offers `"card"`, `"iban"`, etc.
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsElements` factory.
    /// - `element_type`: e.g. `"payment"`, `"card"`, `"iban"`.
    /// - `options`: JSON settings for the element (fields, layout).
    ///
    /// # Returns
    ///
    /// - `Ok(JsPaymentElement)`: the element handle on success.
    /// - `Err(JsValue)`: JS exception for invalid type or options.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let payment_el = elements.create("payment", js_opts).unwrap();
    /// ```
    #[wasm_bindgen(method, catch, js_name = create)]
    pub fn create_element(
        this: &JsElements,
        element_type: &str,
        options: JsValue,
    ) -> Result<JsPaymentElement, JsValue>;

    /// Mount a Stripe Element into the DOM.
    ///
    /// Calls `paymentElement.mount(selector)` in JS.
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsPaymentElement`.
    /// - `selector`: a CSS selector or element ID (e.g. `"#payment-element"`).
    ///
    /// # Returns
    ///
    /// - `Ok(())` on successful mount.
    /// - `Err(JsValue)`: JS exception (selector not found, invalid mount).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// payment_el.mount("#payment-element").unwrap();
    /// ```
    #[wasm_bindgen(method, catch, js_name = mount)]
    pub fn mount(this: &JsPaymentElement, selector: &str) -> Result<(), JsValue>;

    /// Unmount a Stripe Element from the DOM.
    ///
    /// Calls `paymentElement.unmount()` in JS, removing internal listeners.
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsPaymentElement`.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on successful unmount.
    /// - `Err(JsValue)`: JS exception if unmount fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// payment_el.unmount().unwrap();
    /// ```

    #[wasm_bindgen(method, catch, js_name = unmount)]
    pub fn unmount(this: &JsPaymentElement) -> Result<(), JsValue>;

    /// Trigger validation on all Elements fields.
    ///
    /// Corresponds to `elements.submit()` in JS, returning a Promise
    /// that resolves if validation passes or rejects with a JS error.
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsElements` instance.
    ///
    /// # Returns
    ///
    /// - `Ok(Promise)`: resolves to `undefined` on success.
    /// - `Err(JsValue)`: JS exception on immediate error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let promise = elements.submit().unwrap();
    /// let _ = wasm_bindgen_futures::JsFuture::from(promise).await?;
    /// ```
    #[wasm_bindgen(method, catch, js_name = submit)]
    pub fn submit(this: &JsElements) -> Result<Promise, JsValue>;

    /// Handle off-session card authentication (3DS/SCA).
    ///
    /// Calls `stripe.handleCardAction(clientSecret)` in JS to complete
    /// required customer authentication steps.
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsStripe` instance.
    /// - `client_secret`: the PaymentIntent client secret string.
    ///
    /// # Returns
    ///
    /// - `Ok(Promise)`: resolves when authentication completes.
    /// - `Err(JsValue)`: JS exception on error (invalid secret, network).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let promise = stripe.handle_card_action(&client_secret).unwrap();
    /// let _ = wasm_bindgen_futures::JsFuture::from(promise).await?;
    /// ```
    #[wasm_bindgen(method, catch, js_name = handleCardAction)]
    pub fn handle_card_action(this: &JsStripe, client_secret: &str) -> Result<Promise, JsValue>;

    /// Confirm a PaymentIntent with provided options.
    ///
    /// Calls `stripe.confirmPayment(opts)` in JS, which may redirect or
    /// return a Promise resolving to an object containing paymentIntent data.
    ///
    /// # Arguments
    ///
    /// - `this`: the `JsStripe` instance.
    /// - `options`: a JSON object with fields:
    ///    - `elements`: the Elements instance.
    ///    - `clientSecret` (optional): PaymentIntent secret.
    ///    - `confirmParams`: additional confirm parameters (e.g. `return_url`).
    ///    - `redirect`: set `"if_required"` to handle SCA automatically.
    ///
    /// # Returns
    ///
    /// - `Ok(Promise)`: resolves with a JS result object `{ paymentIntent, status, ... }`.
    /// - `Err(JsValue)`: JS exception on immediate error.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let opts = js_sys::Object::new();
    /// // Reflect::set(&opts, &..., elements.as_ref()).unwrap();
    /// let promise = stripe.confirm_payment(opts.into()).unwrap();
    /// let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    /// ```
    #[wasm_bindgen(method, catch, js_name = confirmPayment)]
    pub fn confirm_payment(this: &JsStripe, options: JsValue) -> Result<Promise, JsValue>;
}
