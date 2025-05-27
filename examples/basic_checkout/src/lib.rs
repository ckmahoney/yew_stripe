// src/lib.rs
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_stripe::client::{
    ElementsOptions,
    ConfirmPaymentParams,
    mount_payment_element,
    confirm_payment,
    PaymentResult,
};
use yew_stripe::use_stripejs;

// HTTP fetch & JSON parsing
use gloo_net::http::Request;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreatePIResponse {
    client_secret: String,
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<BasicCheckout>::new().render();
}

#[function_component(BasicCheckout)]
fn basic_checkout() -> Html {
    let stripe_ready = use_stripejs();
    let stripe_el    = use_mut_ref(|| None::<(JsValue, JsValue)>);
    let error        = use_state(|| None::<String>);
    let paid         = use_state(|| false);


    // // This is a restricted key on my personal acount. 
    // // It supports { PaymentIntents: write, PaymentMethods: read, SetupIntents: write }
    // // With no guarantee it will work at the time you find it. 
    // // Please visit stripe dashboard to create your own test key if you need customization!
    // let sk = "rk_test_51KUI60DEw04PTNSc0SBuAbmzGTJyeNlLdF4SuQSSlPsyJdte4MucNkKDPloXtpxEThI671A5Ty8jJ0r0TgXw7PYO006rfcLrc1";
    

    // 1) When Stripe.js is loaded ▶️ fetch client_secret & mount
    {
        let stripe_ready = stripe_ready.clone();
        let stripe_el    = stripe_el.clone();
        let error        = error.clone();

        use_effect_with(stripe_ready, move |ready| {
            if *ready {
                spawn_local(async move {
                    let pk = "pk_test_51KUI60DEw04PTNScWne4kC3RDrpxnydTfgx0B4b4EsBJajLDmqT2t79nEj8kZjeMGx2bfI9BZN1zqo2NX6HrGp4u00Rv0S1OYT";
                    let backend = "http://127.0.0.1:2718/create-payment-intent";

                    // 📡 call your mock server
                    let resp = Request::post(backend).send().await;
                    let client_secret = match resp {
                        Ok(r) if r.ok() => {
                            match r.json::<CreatePIResponse>().await {
                                Ok(d) => d.client_secret,
                                Err(e) => {
                                    error.set(Some(format!("Bad JSON: {}", e)));
                                    return;
                                }
                            }
                        }
                        Ok(r) => {
                            error.set(Some(format!("Server error: {}", r.status())));
                            return;
                        }
                        Err(e) => {
                            error.set(Some(format!("Network error: {}", e)));
                            return;
                        }
                    };

                    // 🧩 mount the Stripe Payment Element
                    let opts = ElementsOptions {
                        client_secret: client_secret.into(),
                        appearance: None,
                    };
                    match mount_payment_element(pk, opts, "#payment-element", None).await {
                        Ok((stripe, elements, _)) => {
                            *stripe_el.borrow_mut() = Some((stripe.into(), elements.into()));
                        }
                        Err(err) => error.set(Some(err.message)),
                    }
                });
            }
            || ()
        });
    }

    // 2) On-click → confirm payment
    let on_click = {
        let stripe_el = stripe_el.clone();
        let error     = error.clone();
        let paid      = paid.clone();
        Callback::from(move |_| {
            if let Some((s, e)) = &*stripe_el.borrow() {
                let s = s.clone();
                let e = e.clone();
                let error = error.clone();
                let paid  = paid.clone();
                spawn_local(async move {
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
            <button onclick={on_click} disabled={!stripe_ready || *paid}>
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
