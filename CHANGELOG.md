# Changelog

All notable changes to this project will be documented in this file.

## [1.0.1] – 2025-05-27

### Added
- **Production-ready example app:**  
  The `examples/basic_checkout` was upgraded to a full-featured, highly polished Stripe checkout skeleton, suitable as a drop-in for real-world Yew/Rust apps.
- **Multi-product checkout demo:**  
  Users can select from multiple products, each with dynamic pricing and detail.
- **Robust payment flow:**  
  Handles successful payments and all major Stripe card failure scenarios with user-friendly messaging.
- **Best-practice error handling:**  
  Client displays clear Stripe error codes/messages on failure, with robust fallback for unknown edge cases.
- **Expanded receipt and card info display:**  
  Payment confirmation screen shows product, amount, card brand/last4 when available, and direct receipt link.
- **Integrated test card reference:**  
  UI includes a comprehensive section for valid and invalid Stripe test cards, fully styled and responsive, with "click to copy" behavior.
- **Accessible and responsive design:**  
  All UI built with semantic HTML and Tailwind CSS, ready for mobile and desktop.
- **Example server improvements:**  
  Demo backend reads the dynamic amount from client, expands payment method details, and supports full Stripe test/failure flow.

### Changed
- **Polished overall UX:**  
  Improved whitespace, typography, layout, and visual hierarchy for a clean, unified checkout experience.
- **Example code refactored:**  
  All components are now modular, reusable, and easily copy-pasteable into other Yew apps.
- **Documentation improved:**  
  Example and code comments clarify how to adapt for new products, endpoints, or custom Stripe flows.

### Fixed
- **Correctly displays test card info and payment details:**  
  No more hardcoded values; product and card data shown always matches the transaction.
- **Reliable error and edge-case handling:**  
  Unknown or missing card info gracefully degrades with helpful, user-friendly messaging.

## [0.2.2] – 2025-05-01

### Fixed
- Standardized wasm-bindgen bindings in `stripe_bindings.rs`: prefixed raw types with `Js` (e.g. `JsStripe`, `JsElements`, `JsPaymentElement`) and enforced consistent `js_name` and `catch` annotations.

## [0.2.1] – 2025-04-30

### Added
- `StripeClient` struct to encapsulate your publishable key and avoid passing it on every call.
- `handle_card_action()` helper for manual off-session SCA/3DS flows.
- `serde_error_to_stripe_error()` to convert serialization errors into `StripeError`.

## [0.2.0] – 2025-04-30

### Added
- `unmount_payment_element()` to support repeated payments in the same session.
- `save_payment_method` flag in `ConfirmPaymentParams` for optional card saving.
- Example Yew app (`examples/basic_checkout`) demonstrating happy-path checkout.

### Changed
- Switched to serde-based `JsValue` conversion for all options.
- Improved error handling in `confirm_payment` to deserialize JS errors into `StripeError`.

## [0.1.0] – 2025-04-30

### Added
- `stripe_interop.rs`: custom Yew hook (`use_stripejs()`) to load Stripe.js v3.
- `bindings.rs`: wasm-bindgen bindings for `Stripe`, `Elements`, `PaymentElement`, `confirmPayment`, etc.
- `client.rs`: high-level async API (`mount_payment_element()`, `validate_payment_element()`, `confirm_payment()`).
- Initial README, CONTRIBUTING, and project scaffolding.


### Fixed
- None (initial release).
