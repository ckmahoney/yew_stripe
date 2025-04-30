# yew-stripe

A small Rust/WASM library for integrating Stripe’s Payment Element into Yew applications—**UI only**, no server logic.

## Why yew-stripe?

Building a modern payment UI in Yew requires embedding Stripe.js and wiring up its Elements API. yew-stripe:

- **Eliminates inline JavaScript** by providing wasm-bindgen bindings and a Yew hook to load Stripe.js on demand.  
- **Offers a pure-Rust API** for mounting the Payment Element, validating inputs, and confirming payments (with SCA/3DS support).  
- **Ships with examples** showing a happy-path checkout in a tiny Yew app.

## Features

- **Dynamic script loader** via `use_stripejs()` hook—injects Stripe.js v3 exactly once.  
- **Low-level bindings** (`bindings.rs`) to `Stripe()`, `elements()`, `create("payment")`, `mount()`, `submit()`, `confirmPayment()`, and `handleCardAction()`.  
- **High-level client** (`client.rs`) exposing:
  - `ElementsOptions` & `PaymentElementOptions` for configuration  
  - `mount_payment_element()` to initialize & mount  
  - `validate_payment_element()` to pre-validate forms  
  - `confirm_payment()` for one-step & two-step flows, with `redirect: if_required` and “save payment method” support  
  - `unmount_payment_element()` for multi-payment scenarios  
- **Example app** (`examples/basic_checkout`) demonstrating a simple “Pay Now” button.

## Constraints & Notes

- **UI Only**: No server-side code; you must create a PaymentIntent on your own backend and pass its client secret to the frontend.  
- **One-time Payments**: Supports “save payment method” if your PaymentIntent is created with `setup_future_usage`.  
- **Yew-Only**: Designed for Yew apps; no support for other frameworks out of the box.  
- **WASM & Trunk**: Requires a build pipeline supporting Rust→WASM (e.g. `trunk` or `wasm-pack + webpack`).  

## Quickstart

1. **Add to Cargo.toml**  
   ```toml
   yew-stripe = { git = "https://github.com/ckmahoney/yew-stripe.git" }
   ```

2. **Load Stripe.js in your Yew component**  
   ```rust
   let stripe_ready = yew_stripe::use_stripejs();
   ```

3. **Mount Payment Element**  
   ```rust
   let (stripe, elements, payment_el) = mount_payment_element(
       "pk_test_…", 
       ElementsOptions { client_secret: cs.into(), appearance: None }, 
       "#payment-element", 
       None
   ).await?;
   ```

4. **Confirm Payment**  
   ```rust
   match confirm_payment(&stripe, &elements, ConfirmPaymentParams { return_url: None, save_payment_method: None, extra: None }, None, true).await {
     PaymentResult::Success(pi) => /* show success */,
     PaymentResult::Error(err)   => /* show error */,
   }
   ```

5. **See the full example** in [`examples/basic_checkout`](./examples/basic_checkout).

## License

[MIT](./LICENSE) © Cortland Mahoney
