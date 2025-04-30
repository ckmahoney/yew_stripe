# Contributing to yew-stripe

First off, thank you for your interest! This is a solo-maintained project, and while contributions are welcome, they are best-effort and not guaranteed to be supported long-term.

## How to Contribute

1. **Fork the repo** and create your feature branch:
   ```bash
   git checkout -b feature/my-cool-feature
   ```
2. **Write code** that aligns with the existing style:
   - Rust 2021 edition.
   - Modules documented with `//!` and `///` comments.
   - Follow `bindings.rs` & `client.rs` patterns for wasm-bindgen and Serde.
3. **Add tests** or examples where appropriate:
   - Unit tests for serialization, error handling, etc.
   - Example Yew apps in `examples/` for new features.
4. **Update docs**:
   - `README.md` for user-facing changes.
   - `CHANGELOG.md` with a new version entry under “Unreleased.”
5. **Open a Pull Request** against `main`:
   - Describe the problem you’re solving.
   - Provide before/after code snippets or screenshots for UI changes.
   - Link any relevant issues.

## AI Co-Development & Prompting Guidelines

This project welcomes AI-assisted contributions. To get the best results from language models, include prompts that:

- **Frame the project context**  
  - Example:  
    > “I’m building an open-source Rust/WASM library named `yew-stripe` for Yew apps. It’s **UI-only**, handles Stripe Payment Element integration, and must be documented for global maintainers.”

- **Specify coding conventions**  
  - Rust 2021 edition, `wasm-bindgen` externs, Serde for JSON, Yew hooks for effects.  
  - Use `//!` crate-level docs and `///` docblocks on public functions and types.

- **Enforce UI/UX & security best practices**  
  - No inline JS strings—use typed externs.  
  - Follow Stripe Elements accessibility and responsive design guidelines.  
  - Validate inputs, handle errors gracefully, never expose secret keys.

- **Outline code structure**  
  - Modules: `bindings.rs`, `client.rs`, `stripe_interop.rs`.  
  - Examples in `examples/basic_checkout`.

- **Encourage co-development**  
  - Prompt for tests, examples, and docs alongside code.  
  - Ask AI to produce clear commit messages and PR descriptions.

### Sample AI Prompt

> “You are an experienced Rust+Yew developer. Generate a `bindings.rs` file for `yew-stripe` that defines wasm-bindgen externs for Stripe.js v3’s PaymentElement API. Use `#[wasm_bindgen]`, Serde, and include thorough `///` documentation. Ensure accessibility and security best practices.”


## Issue Tracking

- **Bugs & small fixes**: file an issue with a minimal reproducible example.
- **Feature requests**: please describe your use case in detail.

> ⚠️ This library is maintained by a single IC; PRs may sit unmerged during busy periods.

## Code of Conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you agree to abide by its terms.

---

Thank you for helping improve yew-stripe! 🎉
