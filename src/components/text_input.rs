use dioxus::html::completions::CompleteWithBraces::i;
use dioxus::prelude::*; // Import KeyboardEvent
use serde::{Deserialize, Serialize};

// Define the Props struct for your TextInput component
#[derive(Props, PartialEq, Clone)]
pub struct TextInputProps {
    #[props(default="Ask anything...".to_string())] // Optional default for placeholder
    placeholder: String,
    // The prop is an EventHandler that takes a KeyboardEvent (specifically for 'Enter')
    on_save: EventHandler<String>,
}


#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    let mut current_input = use_signal(|| String::new());
    let mut show_model_menu = use_signal(|| false);
    let mut show_persona_menu = use_signal(|| false);

    rsx! {
         div {
                            class: "flex-1 flex items-center rounded-lg overflow-hidden",
                            style: "background: #383a40; border: 2px solid #5865f2;",
                            
                            input {
                                class: "flex-1 px-4 py-3 text-sm focus:outline-none bg-transparent",
                                style: "color: #dbdee1; font-size: 16px;",
                                placeholder: props.placeholder.clone(),
                                value: "{current_input}",
                                r#type: "text",
                                autocomplete: "off",
                                spellcheck: "false",
                                enterkeyhint: "done",
                                inputmode: "text",
                                oninput: move |e| current_input.set(e.value()),
                                onfocus: move |_| {
                                    // Close menus when input is focused
                                    show_model_menu.set(false);
                                    show_persona_menu.set(false);
                                },
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter {
 let input = current_input();
                                        if input.trim().is_empty() { return; }
                                        let msg = input.trim().to_string();
                                        props.on_save.call(msg);
                                        current_input.set(String::new());
                                    }
                                }
                            }
                            
                            // Send button (integrated into input field)
                            if !current_input().trim().is_empty() {
                                button {
                                    class: "px-4 py-3 flex items-center justify-center transition-colors hover:bg-opacity-80",
                                    style: "background: #5865f2; color: white; font-size: 16px; border-radius: 0;",
                                    onclick: move |_| {
                                        let input = current_input();
                                        if input.trim().is_empty() { return; }
                                        let msg = input.trim().to_string();
                                        props.on_save.call(msg);
                                        current_input.set(String::new());
                                    },
                                    "→"
                                }
                            }
                        }
    }
}