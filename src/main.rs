#![allow(non_snake_case)]

use std::time::Duration;

use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder, ExtensionOptions,
    Plugins, RenderOptions,
};
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
            .with_source(LlamaSource::qwen_2_5_1_5b_instruct())
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
                                        text: current_message,
                                        response_time: None,
                                        tokens: 0,
                                    });
                                    assistant_responding.set(true);
                                    let assistant_response = MessageState {
                                        user: User::Assistant,
                                        text: String::new(),
                                        response_time: None,
                                        tokens: 0,
                                    };
                                    messages_mut.push(assistant_response);
                                }
                                spawn(async move {
                                    if let Ok(chat) = &mut *chat.write() {
                                        let mut stream = chat.add_message(final_message);
                                        let start = std::time::Instant::now();
                                        while let Some(new_text) = stream.next().await {
                                            let mut messages = messages.write();
                                            let Some(last_message) = messages.last_mut() else { break };
                                            last_message.text += &new_text;
                                            last_message.tokens += 1;
                                        }
                                        let response_time = start.elapsed();
                                        let mut messages = messages.write();
                                        let Some(last_message) = messages.last_mut() else { return };
                                        last_message.response_time = Some(response_time);
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

#[derive(PartialEq, Clone, Copy)]
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
    response_time: Option<Duration>,
    tokens: usize,
}

#[component]
fn Message(message: ReadOnlySignal<MessageState>) -> Element {
    let assistant_placeholder = use_memo(move || {
        let message = message.read();
        message.user == User::Assistant && message.text.is_empty()
    });
    let user = use_memo(move || message.read().user);
    let contents = use_memo(move || {
        let message = message();
        let text = &message.text;
        let mut plugins = Plugins::default();

        let adapter = SyntectAdapterBuilder::new()
            .theme("base16-ocean.dark")
            .build();
        plugins.render.codefence_syntax_highlighter = Some(&adapter);
        let mut extension = ExtensionOptions::default();
        extension.strikethrough = true;
        extension.tagfilter = true;
        extension.table = true;
        extension.autolink = true;

        let mut render = RenderOptions::default();
        render.hardbreaks = true;
        render.github_pre_lang = true;

        let options = comrak::Options {
            extension,
            render,
            ..Default::default()
        };

        println!("{}", text);

        markdown_to_html_with_plugins(text, &options, &plugins)
    });
    let tokens_per_second = use_memo(move || {
        let message = message.read();
        message.response_time.map(|response_time| {
            let tokens = message.tokens;
            let seconds = response_time.as_secs_f64();
            tokens as f64 / seconds
        })
    });

    rsx! {
        div {
            class: "flex flex-row space-x-4",
            div {
                class: "w-2/3 p-2 bg-white rounded-lg shadow-md overflow-y-hidden overflow-x-scroll",
                class: if user() == User::Assistant {
                    "self-start"
                } else {
                    "self-end"
                },
                class: if assistant_placeholder() {
                    "text-gray-400"
                },
                background_color: user().background_color(),
                if assistant_placeholder() {
                    "Thinking..."
                } else {
                    div {
                        dangerous_inner_html: contents
                    }
                }
            }
            if let Some(tokens_per_second) = tokens_per_second() {
                div {
                    class: "text-right",
                    "{tokens_per_second:02.0} tokens/s"
                }
            }
        }
    }
}
