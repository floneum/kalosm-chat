#![allow(non_snake_case)]

use dioxus_router::prelude::*;

use dioxus::{prelude::*, html::input_data::keyboard_types::Key};
use dioxus_signals::{use_signal, Signal};
use kalosm::language::*;

fn main() {
    dioxus_desktop::launch_cfg(
        app,
        dioxus_desktop::Config::new()
            .with_custom_head(r#"<link rel="stylesheet" href="public/tailwind.css">"#.to_string()),
    );
}

fn app(cx: Scope) -> Element {
    render! {
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Setup {},
    #[route("/chat/:assistant_description")]
    Home {
        assistant_description: String,
    },
}

#[component]
fn Setup(cx: Scope) -> Element {
    let assistant_description = use_signal(cx, String::new);
    let navigator = use_navigator(cx);

    render! {
        div {
            class: "flex flex-col h-screen bg-slate-300",

            div {
                class: "flex flex-col flex-1 p-4 space-y-4 overflow-y-auto",

                div {
                    class: "flex flex-col space-y-4",

                    label {
                        class: "text-xl font-bold",
                        "Assistant Description"
                    }

                    input {
                        class: "p-2 bg-white rounded-lg shadow-md",
                        placeholder: "Type a description...",
                        value: "{assistant_description}",
                        oninput: move |event| {
                            assistant_description.set(event.value.clone())
                        },
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                navigator.push(Route::Home {
                                    assistant_description: assistant_description().clone(),
                                });
                            }
                        },
                    }

                    button {
                        class: "p-2 bg-white rounded-lg shadow-md",
                        onclick: move |_| {
                            let assistant_description = assistant_description().clone();
                            navigator.push(Route::Home {
                                assistant_description: if assistant_description.is_empty() {
                                    "Always assist with care, respect, and truth. Respond with utmost utility yet securely. Avoid harmful, unethical, prejudiced, or negative content. Ensure replies promote fairness and positivity.".to_string()
                                } else {
                                    assistant_description
                                },
                            });
                        },
                        "Start Chatting"
                    }
                }
            }
        }
    }
}

#[component]
fn Home(cx: Scope, assistant_description: String) -> Element {
    let current_message = use_signal(cx, String::new);
    let messages: Signal<Vec<Signal<Message>>> = use_signal(cx, Vec::new);
    let assistant_responding = use_signal(cx, || false);
    let model = cx.use_hook(Llama::new_chat);
    let chat = use_signal(cx, || {
        kalosm::Chat::builder(model).with_system_prompt(assistant_description.clone()).build()
    });

    render! {
        div {
            class: "flex flex-col h-screen bg-slate-300",

            div {
                class: "flex flex-col flex-1 p-4 space-y-4 overflow-y-auto",

                for message in messages().iter().copied() {
                    Message {
                        message: message,
                    }
                }

                div {
                    class: "flex flex-row space-x-4",

                    input {
                        class: "flex-1 p-2 bg-white rounded-lg shadow-md",
                        placeholder: "Type a message...",
                        value: "{current_message}",
                        oninput: move |event| {
                            if !*assistant_responding() {
                                current_message.set(event.value.clone())
                            }
                        },
                        onkeydown: move |event| {
                            if !*assistant_responding() && event.key() == Key::Enter {
                                let mut current_message = current_message.write();
                                let mut messages = messages.write();
                                messages.push(Signal::new(Message {
                                    user: User::User,
                                    text: current_message.clone()
                                }));
                                let final_message = current_message.clone();
                                assistant_responding.set(true);
                                let assistant_response = Signal::new(Message {
                                    user: User::Assistant,
                                    text: String::new(),
                                });
                                messages.push(assistant_response);
                                cx.spawn(async move {
                                    let mut chat = chat.write();
                                    if let Ok(mut stream) = chat.add_message(final_message).await {
                                        while let Some(new_text) = stream.next().await {
                                            assistant_response.write().text += &new_text;
                                        }
                                    }
                                    assistant_responding.set(false);
                                });
                                current_message.clear();
                            }
                        },
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone)]
enum User {
    Assistant,
    User,
}

impl User {
    fn background_color(&self) -> &'static str {
        match self {
            User::Assistant => "bg-red-500",
            User::User => "bg-blue-500",
        }
    }
}

#[derive(PartialEq, Clone)]
struct Message {
    user: User,
    text: String,
}

#[component]
fn Message(cx: Scope, message: Signal<Message>) -> Element {
    let message = message();
    let align = if message.user == User::Assistant {
        "self-start"
    } else {
        "self-end"
    };
    let text = &message.text;
    let assistant_placeholder = message.user == User::Assistant && text.is_empty();
    let text = if assistant_placeholder {
        "Thinking..."
    } else {
        text
    };

    let text_color = if assistant_placeholder {
        "text-gray-400"
    } else {
        ""
    };
    render! {
        div {
            class: "w-2/3 p-2 bg-white rounded-lg shadow-md {align} {text_color}",
            background_color: message.user.background_color(),
            "{text}"
        }
    }
}
