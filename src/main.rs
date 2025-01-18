#![allow(non_snake_case)]

use dioxus::{html::input_data::keyboard_types::Key, prelude::*, CapturedError};
use kalosm::language::*;

fn main() {
    launch(app);
}

fn app() -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("public/tailwind.css"),
        }
        document::Link {
            rel: "stylesheet",
            href: asset!("public/loading.css"),
        }
        ErrorBoundary {
            handle_error: |error| rsx! {
                div {
                    class: "flex flex-col h-screen bg-slate-300",
                    "{error:#?}"
                }
            },
            SuspenseBoundary {
                fallback: |_| rsx! {
                    div {
                        class: "w-screen h-screen flex flex-col items-center justify-center",
                        div {
                            class: "spinner",
                        }
                    }
                },
                Router::<Route> {}
            }
        }
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Setup {},
    #[route("/chat/:assistant_description")]
    Home { assistant_description: String },
}

#[component]
fn Setup() -> Element {
    let mut assistant_description = use_signal(String::new);
    let navigator = use_navigator();

    rsx! {
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
                            assistant_description.set(event.value())
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
fn Home(assistant_description: ReadOnlySignal<String>) -> Element {
    let mut current_message = use_signal(String::new);
    let mut messages: Signal<Vec<MessageState>> = use_signal(Vec::new);
    let mut assistant_responding = use_signal(|| false);
    let model = use_resource(|| async move {
        Llama::builder()
            .with_source(LlamaSource::qwen_2_5_0_5b_instruct())
            .build()
            .await
    })
    .suspend()?;
    let mut chat: Signal<Result<Chat<Llama>, CapturedError>> = use_signal(move || {
        let read = model.read();
        match &*read {
            Ok(model) => Ok(model.chat().with_system_prompt(assistant_description())),
            Err(e) => Err(CapturedError::from_display(e.to_string())),
        }
    });

    rsx! {
        div {
            class: "flex flex-col h-screen bg-slate-300",

            div {
                class: "flex flex-col flex-1 p-4 space-y-4 overflow-y-auto",

                for message in messages.read().iter().cloned() {
                    Message {
                        message,
                    }
                }

                div {
                    class: "flex flex-row space-x-4",

                    input {
                        class: "flex-1 p-2 bg-white rounded-lg shadow-md",
                        placeholder: "Type a message...",
                        value: "{current_message}",
                        oninput: move |event| {
                            if !assistant_responding() {
                                current_message.set(event.value())
                            }
                        },
                        onkeydown: move |event| {
                            if !assistant_responding() && event.key() == Key::Enter {
                                let current_message = current_message.take();
                                let final_message = current_message.clone();
                                {
                                    let mut messages_mut = messages.write();
                                    messages_mut.push(MessageState {
                                        user: User::User,
                                        text: current_message
                                    });
                                    assistant_responding.set(true);
                                    let assistant_response = MessageState {
                                        user: User::Assistant,
                                        text: String::new(),
                                    };
                                    messages_mut.push(assistant_response);
                                }
                                spawn(async move {
                                    if let Ok(chat) = &mut *chat.write() {
                                        let mut stream = chat.add_message(final_message);
                                        while let Some(new_text) = stream.next().await {
                                            let mut messages = messages.write();
                                            let Some(last_message) = messages.last_mut() else { break };
                                            last_message.text += &new_text;
                                        }
                                    }
                                    assistant_responding.set(false);
                                });
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
struct MessageState {
    user: User,
    text: String,
}

#[component]
fn Message(message: ReadOnlySignal<MessageState>) -> Element {
    let message = message();
    let text = &message.text;
    let assistant_placeholder = message.user == User::Assistant && text.is_empty();

    rsx! {
        div {
            class: "w-2/3 p-2 bg-white rounded-lg shadow-md",
            class: if message.user == User::Assistant {
                "self-start"
            } else {
                "self-end"
            },
            class: if assistant_placeholder {
                "text-gray-400"
            },
            background_color: message.user.background_color(),
            if assistant_placeholder {
                "Thinking..."
            } else {
                "{text}"
            }
        }
    }
}
