# Changelog

All notable changes to this project will be documented in this file.

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
