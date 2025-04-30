//! stripe_interop.rs
//!
//! Custom Yew hook to load Stripe.js v3 at runtime (no inline JS).
//!
//! # Overview
//! This hook, `use_stripejs()`, injects a single
//! `<script id="stripejs-sdk" src="https://js.stripe.com/v3/" defer>`
//! into `<head>` on first use, returns `false` until the
//! script’s `load` event fires, then returns `true`
//! on every subsequent call.
//!
//! # Cargo.toml
//! ```toml
//! yew = "0.21"                          # Yew framework
//! wasm-bindgen = "0.2"                 # For Closure
//! web-sys = { version = "0.3", features = ["Window","Document","HtmlScriptElement"] }
//! js-sys = "0.3"                       # For Reflect
//! ```
//!
//! # Usage
//! ```rust,ignore
//! use yew::prelude::*;
//! use crate::stripe_interop::use_stripejs;
//!
//! #[function_component(App)]
//! fn app() -> Html {
//!     let stripe_ready = use_stripejs();
//!     html! {
//!         if stripe_ready {
//!             <p>{"✅ Stripe.js loaded"}</p>
//!         } else {
//!             <p>{"⏳ Loading Stripe.js..."}</p>
//!         }
//!     }
//! }
//! ```

use yew::prelude::*;
use yew::functional::hook; // required for custom hooks marked #[hook]
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::js_sys::Reflect;
use web_sys::HtmlScriptElement;

/// Custom hook: load Stripe.js v3 exactly once and track readiness.
///
/// # Returns
/// - `false` while the `<script>` is being fetched & parsed.
/// - `true` once `window.Stripe` exists (script loaded & parsed).
///
/// All components using `use_stripejs()` will share the same script
/// insertion logic and state.
#[hook]
pub fn use_stripejs() -> bool {
    // Initialize state: check if `window.Stripe` already present
    let loaded = use_state(|| {
        web_sys::window()
            .and_then(|win| {
                // Reflect.has(window, "Stripe") → Result<bool, _>
                Reflect::has(&win, &JsValue::from_str("Stripe"))
                    .ok()             // Result<bool, _> → Option<bool>
                    .filter(|&b| b)  // keep only `true`
            })
            .map(|_| true)         // Some(true) → Some(true)
            .unwrap_or(false)      // None → false
    });

    {
        let loaded = loaded.clone(); // UseStateHandle<bool> is Clone + Deref&#8203;:contentReference[oaicite:6]{index=6}
        use_effect(move || {
            // If not yet loaded, inject the Stripe.js script once
            if !*loaded {
                let document = web_sys::window()
                    .expect("no window")
                    .document()
                    .expect("no document");

                // Only inject if `<script id="stripejs-sdk">` missing
                if document.get_element_by_id("stripejs-sdk").is_none() {
                    let script: HtmlScriptElement = document
                        .create_element("script")
                        .expect("create script")
                        .dyn_into()
                        .expect("cast script");

                    script.set_id("stripejs-sdk");
                    script.set_src("https://js.stripe.com/v3/");
                    script.set_defer(true);

                    // Closure to run on script.load → set loaded = true
                    let onload_closure = Closure::wrap(Box::new(move || {
                        loaded.set(true);
                    }) as Box<dyn Fn()>); // Closure needs 'static Fn()&#8203;:contentReference[oaicite:7]{index=7}

                    script.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                    onload_closure.forget(); // Leak so it lives until load event

                    document
                        .head()
                        .expect("head missing")
                        .append_child(&script)
                        .expect("append script");
                }
            }
            // No cleanup needed
            || ()
        });
    }

    // Return the current loaded state (derefs UseStateHandle<bool>)&#8203;:contentReference[oaicite:8]{index=8}
    *loaded
}
