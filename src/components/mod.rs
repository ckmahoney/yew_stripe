use yew::prelude::*;
use web_sys::*;


/// A simple, styled button.
#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    /// Button label text
    pub label: String,
    /// Click handler
    pub onclick: Callback<MouseEvent>,
    /// Disable state
    #[prop_or_default]
    pub disabled: bool,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    html! {
        <button
            onclick={props.onclick.clone()}
            disabled={props.disabled}
            class="ygc-button" // you can reference an external stylesheet or styled crate
        >
            { &props.label }
        </button>
    }
}

/// A basic controlled text input.
#[derive(Properties, PartialEq)]
pub struct TextInputProps {
    /// Current value
    pub value: String,
    /// Emits new value on each keystroke
    pub oninput: Callback<String>,
    /// Placeholder text
    #[prop_or_default]
    pub placeholder: String,
}

#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    let oninput = props.oninput.clone();
    html! {
        <input
            type="text"
            class="ygc-text-input"
            value={props.value.clone()}
            placeholder={props.placeholder.clone()}
            oninput={Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                oninput.emit(input.value());
            })}
        />
    }
}
