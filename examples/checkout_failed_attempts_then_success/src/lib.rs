use yew::prelude::*;
use serde::Deserialize;
use yew_stripe::{Stripe, CardElement, PaymentIntent, StripeError};

/// A struct used to deserialize the PaymentIntent creation API response.
#[derive(Debug, Deserialize)]
struct CreatePaymentIntentResponse {
    /// The client secret of the created PaymentIntent, to be used with Stripe.js confirmation.
    client_secret: String,
}

/// Initiates the PaymentIntent creation process by sending a POST request to the specified API URL.
/// 
/// # Arguments
/// 
/// * `api_url` - The endpoint URL of the backend API that creates a PaymentIntent and returns its client secret.
/// 
/// # Returns
/// 
/// A Result containing the PaymentIntent client secret on success, or an error message `String` on failure.
/// 
/// # Details
/// 
/// This function uses a conventional HTTP POST request with JSON headers to contact the backend. It expects
/// the backend to return JSON in the form `{ "client_secret": "..." }`. The `FetchArgs` used in this request
/// include the URL, method (POST), appropriate headers, and an empty JSON body for simplicity.
/// 
/// For example, the request is structured as:
/// ```ignore
/// FetchArgs { 
///     url: api_url, 
///     method: POST, 
///     headers: { "Content-Type": "application/json" }, 
///     body: "{}" 
/// }
/// ```
/// 
/// In a real application, you might include details like the amount or currency in the body.
/// Here we assume the backend uses default or preset values for demonstration.
async fn init_payment_intent_flow(api_url: &str) -> Result<String, String> {
    // Build and send the HTTP request to create a PaymentIntent.
    let response = gloo_net::http::Request::post(api_url)
        .header("Content-Type", "application/json")
        .body("{}") // sending an empty JSON body as a placeholder
        .send().await;
    // Handle the HTTP response.
    match response {
        Ok(resp) => {
            if resp.status() != 200 {
                // Non-200 HTTP status is treated as an error.
                return Err(format!("API request failed with status: {}", resp.status()));
            }
            // Parse the JSON body to extract the client secret.
            let body_text = resp.text().await.map_err(|err| err.to_string())?;
            let data: CreatePaymentIntentResponse = serde_json::from_str(&body_text)
                .map_err(|err| err.to_string())?;
            Ok(data.client_secret)
        }
        Err(err) => {
            // Network or request error.
            Err(format!("Network error: {}", err))
        }
    }
}

/// Represents the current status of the payment flow.
#[derive(Debug)]
enum PaymentStatus {
    /// No payment attempt has been made yet.
    Idle,
    /// A payment attempt is in progress.
    Processing,
    /// The last payment attempt failed with an error.
    Error(PaymentError),
    /// Payment succeeded.
    Success(PaymentIntentInfo),
}

/// Contains details about a payment error for display and debugging.
#[derive(Debug)]
struct PaymentError {
    /// A Stripe error code identifying the type of error (if available).
    code: Option<String>,
    /// A human-readable error message describing what went wrong.
    message: String,
}

/// Basic information about a successful PaymentIntent (for demonstration purposes).
#[derive(Debug)]
struct PaymentIntentInfo {
    /// The Stripe PaymentIntent ID.
    id: String,
    /// The amount of the PaymentIntent in the smallest currency unit (e.g., cents).
    amount: u64,
    /// The currency of the PaymentIntent (ISO 4217 currency code).
    currency: String,
}

/// A Yew component implementing a checkout flow with multiple payment attempts using Stripe Elements.
/// 
/// This component showcases two failed payment attempts (insufficient funds and incorrect card number)
/// followed by a successful payment. It guides the user through each step, displaying relevant error
/// messages and allowing retries with new card details.
struct CheckoutForm {
    /// The client secret for the PaymentIntent, obtained from the backend.
    client_secret: Option<String>,
    /// Current status of the payment process (idle, processing, error, or success).
    status: PaymentStatus,
    /// The Stripe object used to handle Elements and payment confirmation.
    stripe: Stripe,
    /// The Card Element representing the credit card input field.
    card_element: Option<CardElement>,
    /// Reference to the DOM node where the Card Element is mounted.
    card_mount_node: NodeRef,
    /// API endpoint URL for creating the PaymentIntent.
    api_url: String,
}

/// Messages used to update the state of the `CheckoutForm` component.
enum Msg {
    /// Trigger creating a new PaymentIntent via the backend API.
    CreatePaymentIntent,
    /// Indicates the PaymentIntent client secret was successfully retrieved.
    PaymentIntentReady(String),
    /// Attempts to confirm the payment using the provided card details.
    SubmitPayment,
    /// Final result of a payment attempt (either success or failure).
    PaymentFinished(Result<PaymentIntent, StripeError>),
}

/// Properties for the `CheckoutForm` component.
#[derive(Properties, PartialEq)]
struct CheckoutProps {
    /// The backend API URL for creating a PaymentIntent. If not provided, a default is used.
    #[prop_or_default]
    api_url: String,
}

impl Component for CheckoutForm {
    type Message = Msg;
    type Properties = CheckoutProps;

    fn create(ctx: &Context<Self>) -> Self {
        // Initialize the Stripe.js object with your publishable key.
        // In practice, set STRIPE_PUBLISHABLE_KEY to your Stripe publishable API key.
        const STRIPE_PUBLISHABLE_KEY: &str = "pk_test_YOUR_PUBLISHABLE_KEY";
        let stripe = Stripe::new(STRIPE_PUBLISHABLE_KEY);

        // Determine the API URL from props or use a default placeholder.
        let api_url = if !ctx.props().api_url.is_empty() {
            ctx.props().api_url.clone()
        } else {
            // Default to a placeholder URL; replace with a real endpoint as needed.
            "https://example.com/create-payment-intent".to_string()
        };

        Self {
            client_secret: None,
            status: PaymentStatus::Idle,
            stripe,
            card_element: None,
            card_mount_node: NodeRef::default(),
            api_url,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // On the first render, mount the Stripe Card Element into the DOM.
            let elements = self.stripe.elements();
            let card = elements.create_card();
            if let Some(card_container) = self.card_mount_node.cast::<web_sys::HtmlElement>() {
                card.mount(card_container);
            }
            self.card_element = Some(card);
            // Initiate the creation of a PaymentIntent by calling the backend API.
            ctx.link().send_message(Msg::CreatePaymentIntent);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::CreatePaymentIntent => {
                // Start an asynchronous task to fetch the PaymentIntent client secret.
                let api = self.api_url.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match init_payment_intent_flow(&api).await {
                        Ok(secret) => link.send_message(Msg::PaymentIntentReady(secret)),
                        Err(error_msg) => {
                            // If we failed to get a PaymentIntent, treat it as an error state.
                            link.send_message(Msg::PaymentFinished(Err(StripeError {
                                code: None,
                                message: error_msg,
                            })));
                        }
                    }
                });
                // While waiting for the response, we can show a loading state (client_secret is None).
                // Return false to avoid re-rendering until the fetch completes.
                false
            }
            Msg::PaymentIntentReady(secret) => {
                // Store the client secret and render the form now that we can proceed.
                self.client_secret = Some(secret);
                true // re-render to display the card form and pay button
            }
            Msg::SubmitPayment => {
                if let (Some(secret), Some(card)) = (&self.client_secret, &self.card_element) {
                    // Update state to processing (e.g., disable the pay button).
                    self.status = PaymentStatus::Processing;
                    let stripe = self.stripe.clone();
                    let card_clone = card.clone();
                    let secret_clone = secret.clone();
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        // Use Stripe.js to confirm the card payment with the client secret.
                        let result = stripe.confirm_card_payment(&secret_clone, &card_clone).await;
                        link.send_message(Msg::PaymentFinished(result));
                    });
                    true // re-render (button text changes to "Processing...")
                } else {
                    // No client secret or card element available (this shouldn't happen in normal flow).
                    false
                }
            }
            Msg::PaymentFinished(result) => {
                // Payment attempt completed (either success or failure).
                match result {
                    Ok(payment_intent) => {
                        // On success, update status with PaymentIntent info.
                        self.status = PaymentStatus::Success(PaymentIntentInfo {
                            id: payment_intent.id,
                            amount: payment_intent.amount,
                            currency: payment_intent.currency,
                        });
                    }
                    Err(err) => {
                        // On failure, update status with error information.
                        self.status = PaymentStatus::Error(PaymentError {
                            code: err.code.clone(),
                            message: err.message.clone(),
                        });
                        // Clear the card input field so the user can retry with new details.
                        if let Some(card) = &self.card_element {
                            card.clear();
                        }
                    }
                }
                true // re-render to show error or success message
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_pay_click = ctx.link().callback(|_| Msg::SubmitPayment);
        let payment_success = matches!(self.status, PaymentStatus::Success(_));

        html! {
          <div class="checkout-form">
            <h1>{ "Yew Stripe Checkout: Failed Attempts then Success" }</h1>
            // Instructional text to guide the user through the demo.
            <p>{ "This example simulates two failed payments followed by a successful one. Use the card numbers below in order:" }</p>
            <ul>
              <li>{ "First attempt – use card 4000 0000 0000 9995 to simulate insufficient funds&#8203;:contentReference[oaicite:3]{index=3}." }</li>
              <li>{ "Second attempt – use card 4242 4242 4242 4241 to simulate an invalid card number&#8203;:contentReference[oaicite:4]{index=4}." }</li>
              <li>{ "Third attempt – use card 4242 4242 4242 4242 to simulate a successful payment&#8203;:contentReference[oaicite:5]{index=5}." }</li>
            </ul>

            // If the PaymentIntent is not yet ready, show a loading indicator.
            {
                if self.client_secret.is_none() {
                    html! { <p>{ "Initializing payment intent..." }</p> }
                } else {
                    // Otherwise, show the payment form (card field and Pay button).
                    html! {
                        <div class="payment-form">
                            <div>
                                <label for="card-element">{ "Card Details:" }</label>
                                <div id="card-element" ref={self.card_mount_node}></div>
                            </div>
                            // Display any error message from a failed attempt.
                            {
                                if let PaymentStatus::Error(error) = &self.status {
                                    html! {
                                        <p class="error-message" aria-live="polite">
                                            { &error.message }
                                        </p>
                                    }
                                } else {
                                    Html::default()
                                }
                            }
                            <button type="button"
                                onclick={on_pay_click}
                                disabled={matches!(self.status, PaymentStatus::Processing) || payment_success}>
                                {
                                    if matches!(self.status, PaymentStatus::Processing) {
                                        "Processing..."
                                    } else if payment_success {
                                        "Pay"  // Button will be disabled on success, label stays "Pay"
                                    } else {
                                        "Pay"
                                    }
                                }
                            </button>
                        </div>
                    }
                }
            }

            // If payment succeeded, display a confirmation message.
            {
                if let PaymentStatus::Success(info) = &self.status {
                    html! {
                        <p class="success-message">
                            { "Payment succeeded! " }
                            { format!("PaymentIntent {} for {:.2} {} was confirmed.", 
                                      info.id, info.amount as f64 / 100.0, info.currency.to_uppercase()) }
                        </p>
                    }
                } else {
                    Html::default()
                }
            }
          </div>
        }
    }
}

// Entry point: mount the CheckoutForm component in the page.
fn main() {
    yew::Renderer::<CheckoutForm>::new().render();
}
