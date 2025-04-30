use yew::prelude::*;
use yew_stripe_interop::use_stripejs;
use yew_stripe::{ElementsOptions, ConfirmPaymentParams, mount_payment_element, confirm_payment, PaymentResult};

#[function_component(BasicCheckout)]
fn basic_checkout() -> Html {
    // 1) Load Stripe.js
    let stripe_ready = use_stripejs();

    // 2) Hold Stripe & Elements instances
    let stripe_el = use_mut_ref(|| None::<(wasm_bindgen::JsValue, wasm_bindgen::JsValue)>);
    let error = use_state(|| None::<String>);
    let paid = use_state(|| false);

    // 3) Once script loaded, mount Payment Element
    {
        let stripe_el = stripe_el.clone();
        let error = error.clone();
        use_effect_with_deps(move |ready| {
            if **ready {
                wasm_bindgen_futures::spawn_local(async move {
                    // Replace with your real keys/secret
                    let pk = "pk_test_XXXXXXXXXXXXXXXX";
                    let cs = "pi_client_secret_XXXXXXXXXXXXXXXX";
                    let opts = ElementsOptions { client_secret: cs.into(), appearance: None };
                    match mount_payment_element(pk, opts, "#payment-element", None).await {
                        Ok((stripe, elements, _pe)) => {
                            *stripe_el.borrow_mut() = Some((stripe.into(), elements.into()));
                        }
                        Err(err) => error.set(Some(err.message)),
                    }
                });
            }
            || ()
        }, stripe_ready);
    }

    // 4) On Pay Now click → confirm payment
    let on_click = {
        let stripe_el = stripe_el.clone();
        let error = error.clone();
        let paid = paid.clone();
        Callback::from(move |_| {
            if let Some((s, e)) = &*stripe_el.borrow() {
                let s = s.clone();
                let e = e.clone();
                let error = error.clone();
                let paid = paid.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let params = ConfirmPaymentParams {
                        return_url: None,
                        save_payment_method: None,
                        extra: None,
                    };
                    match confirm_payment(&s.into(), &e.into(), params, None, true).await {
                        PaymentResult::Success(_) => paid.set(true),
                        PaymentResult::Error(err) => error.set(Some(err.message)),
                    }
                });
            }
        })
    };

    html! {
        <div>
            <div id="payment-element" style="margin-bottom:1rem;"></div>
            <button {on_click} disabled={!stripe_ready || *paid}>
                { if *paid { "🎉 Paid" } else { "Pay Now" } }
            </button>
            {
                if let Some(msg) = &*error {
                    html! { <p style="color:red;">{ msg.clone() }</p> }
                } else {
                    Html::default()
                }
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<BasicCheckout>::new().render();
}
