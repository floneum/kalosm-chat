#![allow(non_snake_case)]
use std::time::Duration;

use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder, ExtensionOptions,
    Plugins, RenderOptions,
};
use dioxus::document::eval;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*, CapturedError};
use kalosm::language::*;

fn main() {
    launch(app);
}

fn app() -> Element {
    rsx! {
        document::Stylesheet {
            href: asset!("/assets/tailwind.css"),
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
                        class: "flex items-center justify-center min-h-screen bg-gradient-to-br from-gray-50 to-gray-200",
                        div {
                            class: "animate-spin rounded-full h-32 w-32 border-t-4 border-b-4 border-[#2B2A28]"
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
    #[route("/chat/:user/:model_id/:file?:assistant_description")]
    Home {
        user: String,
        model_id: String,
        file: String,
        assistant_description: String,
    },
}

#[component]
fn Setup() -> Element {
    let navigator = use_navigator();
    let mut user = use_signal(|| "bartowski".to_string());
    let mut model_id = use_signal(|| "Qwen2.5-7B-Instruct-GGUF".to_string());
    let mut file = use_signal(|| "Qwen2.5-7B-Instruct-Q4_K_M.gguf".to_string());
    let mut assistant_description = use_signal(|| {
        "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.".to_string()
    });
    let mut disabled = use_signal(|| false);
    _ = use_resource(move || async move {
        disabled.set(true);
        _ = reqwest::get(format!("hf://{}/{}/{}", user(), model_id(), file()))
            .await
            .is_ok();
        disabled.set(false);
    });

    let start_chat = move || {
        if disabled() {
            return;
        }
        navigator.push(Route::Home {
            assistant_description: assistant_description(),
            user: user(),
            model_id: model_id(),
            file: file(),
        });
    };

    let mut model_input_mount: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    let mut file_input_mount: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    let mut description_input_mount: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);

    rsx! {
        div {
            class: "flex items-center justify-center min-h-screen bg-gradient-to-br from-gray-50 to-gray-200",
            div {
                class: "w-full max-w-md p-6 space-y-2 bg-white rounded-2xl shadow-2xl",
                div {
                    class: "text-center",
                    h1 {
                        class: "text-2xl font-bold text-gray-800",
                        "Configure Your Assistant"
                    }
                    p {
                        class: "text-sm text-gray-500",
                        "Set the model parameters to start"
                    }
                }

                div {
                    class: "space-y-2",
                    div {
                        label {
                            class: "block text-xs font-medium text-gray-600",
                            "Hugging Face User (Optional)"
                        }
                        input {
                            class: "w-full px-1 py-1 text-sm text-gray-800 bg-transparent border-0 border-b-2 border-gray-200 focus:outline-none focus:ring-0 focus:border-[#2B2A28] transition-colors",
                            placeholder: "e.g., bartowski",
                            value: "{user}",
                            oninput: move |event| user.set(event.value()),
                            onkeydown: move |event| async move {
                                if event.key() == Key::Enter {
                                    if let Some(mount) = model_input_mount() {
                                        _ = mount.set_focus(true).await;
                                    }
                                }
                            },
                        }
                    }

                    div {
                        label {
                            class: "block text-xs font-medium text-gray-600",
                            "Model ID"
                        }
                        input {
                            class: "w-full px-1 py-1 text-sm text-gray-800 bg-transparent border-0 border-b-2 border-gray-200 focus:outline-none focus:ring-0 focus:border-[#2B2A28] transition-colors",
                            placeholder: "e.g., Qwen2.5-7B-Instruct-GGUF",
                            value: "{model_id}",
                            oninput: move |event| model_id.set(event.value()),
                            onkeydown: move |event| async move {
                                if event.key() == Key::Enter {
                                    if let Some(mount) = file_input_mount() {
                                        _ = mount.set_focus(true).await;
                                    }
                                }
                            },
                            onmounted: move |mount| model_input_mount.set(Some(mount.data)),
                        }
                    }

                    div {
                        label {
                            class: "block text-xs font-medium text-gray-600",
                            "Model File"
                        }
                        input {
                            class: "w-full px-1 py-1 text-sm text-gray-800 bg-transparent border-0 border-b-2 border-gray-200 focus:outline-none focus:ring-0 focus:border-[#2B2A28] transition-colors",
                            placeholder: "e.g., Qwen2.5-7B-Instruct-Q4_K_M.gguf",
                            value: "{file}",
                            oninput: move |event| file.set(event.value()),
                            onkeydown: move |event| async move {
                                if event.key() == Key::Enter {
                                    if let Some(mount) = description_input_mount() {
                                        _ = mount.set_focus(true).await;
                                    }
                                }
                            },
                            onmounted: move |mount| file_input_mount.set(Some(mount.data)),
                        }
                    }

                    div {
                        label {
                            class: "block text-xs font-medium text-gray-600",
                            "Assistant Persona"
                        }
                        textarea {
                            class: "w-full px-1 py-1 text-sm text-gray-800 bg-transparent border-0 border-b-2 border-gray-200 focus:outline-none focus:ring-0 focus:border-[#2B2A28] transition-colors resize-none",
                            placeholder: "Describe your assistant's personality...",
                            rows: 2,
                            value: "{assistant_description}",
                            oninput: move |event| assistant_description.set(event.value()),
                            onkeydown: move |event| {
                                if event.key() == Key::Enter && !event.modifiers().shift() {
                                    start_chat();
                                }
                            },
                            onmounted: move |mount| description_input_mount.set(Some(mount.data)),
                        }
                    }

                    button {
                        class: "w-full px-4 py-2 mt-4 font-bold text-white bg-[#2B2A28] rounded-lg hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-[#2B2A28] transition-all duration-300 transform hover:shadow-lg hover:-translate-y-1 disabled:bg-gray-400 disabled:cursor-not-allowed disabled:transform-none disabled:shadow-none",
                        onclick: move |_| start_chat(),
                        disabled: disabled(),
                        if disabled() {
                            "Verifying Model..."
                        } else {
                            "Start Chatting"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Home(
    user: ReadOnlySignal<String>,
    model_id: ReadOnlySignal<String>,
    file: ReadOnlySignal<String>,
    assistant_description: ReadOnlySignal<String>,
) -> Element {
    let mut current_message = use_signal(String::new);
    let mut messages: Signal<Vec<MessageState>> = use_signal(Vec::new);
    let mut assistant_responding = use_signal(|| false);
    let model = use_resource(move || async move {
        Llama::builder()
            .with_source(LlamaSource::new(FileSource::huggingface(
                format!("{user}/{model_id}"),
                "main",
                file,
            )))
            .build()
            .await
    })
    .suspend()?;
    let mut chat: Signal<Result<Chat<Llama>, CapturedError>> = use_signal(move || {
        let read = model.read();
        match &*read {
            Ok(model) => {
                let mut chat = model.chat();
                let assistant_description = assistant_description();
                if !assistant_description.is_empty() {
                    chat = chat.with_system_prompt(assistant_description);
                }
                Ok(chat)
            }
            Err(e) => Err(CapturedError::from_display(e.to_string())),
        }
    });

    use_effect(move || {
        let messages_len = messages.read().len();
        if messages_len > 0 {
            let _ = eval(
                r#"
                setTimeout(() => {
                    const container = document.querySelector('.overflow-y-auto');
                    if (container) {
                        container.scrollTop = container.scrollHeight;
                    }
                }, 100);
            "#,
            );
        }
    });

    rsx! {
        div {
            class: "flex flex-col h-screen bg-gray-100 relative",

            // Minimal reconfigure button in top-right corner
            div {
                class: "absolute top-4 right-4 z-10",
                Link {
                    to: Route::Setup {},
                    class: "p-2 bg-white rounded-full shadow-md hover:shadow-lg transition-shadow duration-200 text-gray-600 hover:text-gray-800",
                    title: "Reconfigure settings",
                    span {
                        class: "text-lg",
                        "⚙️"
                    }
                }
            }

            div {
                class: "flex-1 p-10 pt-20 space-y-4 overflow-y-auto",
                id: "messages-container",
                for message in messages.read().iter().cloned() {
                    Message {
                        message,
                    }
                }
            }

            div {
                class: "p-4 bg-white border-t border-gray-200",
                div {
                    class: "flex flex-row space-x-4",
                    input {
                        class: "flex-1 p-2 bg-white rounded-lg shadow-md focus:outline-none focus:ring-2 focus:ring-[#2B2A28]",
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
            User::Assistant => "bg-gray-200",
            User::User => "bg-[#2B2A28]",
        }
    }

    fn text_color(&self) -> &'static str {
        match self {
            User::Assistant => "text-gray-800",
            User::User => "text-white",
        }
    }

    fn token_color(&self) -> &'static str {
        match self {
            User::Assistant => "text-gray-500",
            User::User => "text-gray-400",
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

    let user = user();

    rsx! {
        div {
            class: "flex flex-row space-x-4",
            class: if user == User::Assistant {
                "justify-start"
            } else {
                "justify-end"
            },
            div {
                class: "max-w-[66.66%] p-4 shadow-lg flex flex-col rounded-3xl rounded-bl-lg",
                class: "{user.background_color()}",
                class: "{user.text_color()}",
                class: if assistant_placeholder() {
                    "text-gray-400"
                },
                div {
                    class: "flex-grow",
                    dangerous_inner_html: "{contents}"
                }
                if let Some(tokens_per_second) = tokens_per_second() {
                    div {
                        class: "text-xs self-end pt-1",
                        class: "{user.token_color()}",
                        "{tokens_per_second:02.0} tokens/s"
                    }
                }
                if assistant_placeholder() {
                    "Thinking..."
                }
            }
        }
    }
}
