#![recursion_limit = "256"]
#![allow(non_snake_case)]

mod components;

use std::time::Duration;
use wasm_timer::Instant;

use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder, ExtensionOptions,
    Plugins, RenderOptions,
};
use components::avatar::{Avatar, AvatarFallback, AvatarImageSize};
use components::button::{Button, ButtonVariant};
use components::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use components::input::Input;
use components::label::Label;
use components::progress::{Progress, ProgressIndicator};
use components::scroll_area::ScrollArea;
use components::textarea::Textarea;
use dioxus::document::eval;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*, CapturedError};
use kalosm_llama::prelude::*;

fn main() {
    launch(app);
}

fn app() -> Element {
    let loading_progress = use_context_provider(|| LoadingProgress {
        loading_progress: Signal::new_maybe_sync(0.0),
    });
    let current_loading_progress = loading_progress.loading_progress.cloned() * 100.0;
    rsx! {
        document::Stylesheet {
            href: asset!("/assets/dx-components-theme.css"),
        }
        ErrorBoundary {
            handle_error: |error| rsx! {
                div {
                    style: "display: flex; flex-direction: column; height: 100vh; background: var(--primary-error-color);",
                    "{error:#?}"
                }
            },
            SuspenseBoundary {
                fallback: move |_| rsx! {
                    div {
                        style: "display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; background: var(--primary-color); gap: 1.5rem;",
                        div {
                            style: "width: 16rem;",
                            Progress {
                                value: current_loading_progress as f64,
                                max: 100.0,
                                ProgressIndicator {}
                            }
                        }
                        p {
                            style: "color: var(--secondary-color-5); font-weight: 500;",
                            "Loading model... {current_loading_progress:.0}%"
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
    let mut user = use_signal(|| "Qwen".to_string());
    let mut model_id = use_signal(|| "Qwen2.5-0.5B-Instruct-GGUF".to_string());
    let mut file = use_signal(|| "qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string());
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

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: center; min-height: 100vh;",
            Card {
                style: "width: 100%; max-width: 28rem;",
                CardHeader {
                    style: "text-align: center;",
                    CardTitle {
                        "Configure Your Assistant"
                    }
                    CardDescription {
                        "Set the model parameters to start"
                    }
                }
                CardContent {
                    div {
                        style: "display: flex; flex-direction: column; gap: 1rem;",
                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            Label {
                                html_for: "hf-user",
                                "Hugging Face User (Optional)"
                            }
                            Input {
                                id: "hf-user",
                                placeholder: "e.g., bartowski",
                                value: "{user}",
                                oninput: move |event: FormEvent| user.set(event.value()),
                            }
                        }

                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            Label {
                                html_for: "model-id",
                                "Model ID"
                            }
                            Input {
                                id: "model-id",
                                placeholder: "e.g., Qwen2.5-7B-Instruct-GGUF",
                                value: "{model_id}",
                                oninput: move |event: FormEvent| model_id.set(event.value()),
                            }
                        }

                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            Label {
                                html_for: "model-file",
                                "Model File"
                            }
                            Input {
                                id: "model-file",
                                placeholder: "e.g., Qwen2.5-7B-Instruct-Q4_K_M.gguf",
                                value: "{file}",
                                oninput: move |event: FormEvent| file.set(event.value()),
                            }
                        }

                        div {
                            style: "display: flex; flex-direction: column; gap: 0.5rem;",
                            Label {
                                html_for: "persona",
                                "Assistant Persona"
                            }
                            Textarea {
                                id: "persona",
                                placeholder: "Describe your assistant's personality...",
                                rows: 3,
                                value: "{assistant_description}",
                                oninput: move |event: FormEvent| assistant_description.set(event.value()),
                                onkeydown: move |event: KeyboardEvent| {
                                    if event.key() == Key::Enter && !event.modifiers().shift() {
                                        start_chat();
                                    }
                                },
                            }
                        }

                        Button {
                            style: "width: 100%; margin-top: 1rem;",
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
}

#[derive(Clone)]
struct LoadingProgress {
    loading_progress: SyncSignal<f32>,
}

#[component]
fn Home(
    user: ReadSignal<String>,
    model_id: ReadSignal<String>,
    file: ReadSignal<String>,
    assistant_description: ReadSignal<String>,
) -> Element {
    let mut loading_progress: LoadingProgress = use_context();
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
            .build_with_loading_handler(move |progress| {
                loading_progress.loading_progress.set(progress.progress())
            })
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
                    const container = document.querySelector('[data-radix-scroll-area-viewport]');
                    if (container) {
                        container.scrollTop = container.scrollHeight;
                    }
                }, 100);
            "#,
            );
        }
    });

    let mut send_message = move || {
        if assistant_responding() {
            return;
        }
        let current_msg = current_message.take();
        if current_msg.is_empty() {
            return;
        }
        let final_message = current_msg.clone();
        {
            let mut messages_mut = messages.write();
            messages_mut.push(MessageState {
                user: ChatUser::User,
                text: current_msg,
                response_time: None,
                tokens: 0,
            });
            assistant_responding.set(true);
            let assistant_response = MessageState {
                user: ChatUser::Assistant,
                text: String::new(),
                response_time: None,
                tokens: 0,
            };
            messages_mut.push(assistant_response);
        }
        spawn(async move {
            match &mut *chat.write() {
                Ok(chat) => {
                    let mut stream = chat.add_message(final_message);
                    let start = Instant::now();
                    while let Some(new_text) = stream.next().await {
                        let mut messages = messages.write();
                        let Some(last_message) = messages.last_mut() else {
                            break;
                        };
                        last_message.text += &new_text;
                        last_message.tokens += 1;
                    }
                    let response_time = start.elapsed();
                    let mut messages = messages.write();
                    let Some(last_message) = messages.last_mut() else {
                        return;
                    };
                    last_message.response_time = Some(response_time);
                }
                Err(err) => {
                    let mut messages = messages.write();
                    let Some(last_message) = messages.last_mut() else {
                        return;
                    };
                    last_message.text = format!("Error: {}", err);
                }
            }
            assistant_responding.set(false);
        });
    };

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; background: var(--primary-color-1);",

            // Header with settings button
            div {
                style: "display: flex; align-items: center; justify-content: space-between; padding: 1rem 1.5rem; background: var(--primary-color); border-bottom: 1px solid var(--primary-color-6);",
                h1 {
                    style: "font-size: 1.125rem; font-weight: 600; color: var(--secondary-color-4);",
                    "Chat"
                }
                Link {
                    to: Route::Setup {},
                    Button {
                        variant: ButtonVariant::Ghost,
                        style: "padding: 0.5rem;",
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "20",
                            height: "20",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path {
                                d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
                            }
                            circle {
                                cx: "12",
                                cy: "12",
                                r: "3"
                            }
                        }
                    }
                }
            }

            // Messages area with ScrollArea
            ScrollArea {
                style: "flex: 1;",
                div {
                    style: "padding: 1.5rem;",
                    id: "messages-container",
                    for message in messages.read().iter().cloned() {
                        Message {
                            message,
                        }
                    }
                }
            }

            // Input area
            Card {
                CardContent {
                    style: "padding: 1rem;",
                    div {
                        style: "display: flex; flex-direction: row; gap: 0.75rem;",
                        Input {
                            style: "flex: 1;",
                            placeholder: "Type a message...",
                            value: "{current_message}",
                            disabled: assistant_responding(),
                            oninput: move |event: FormEvent| {
                                if !assistant_responding() {
                                    current_message.set(event.value())
                                }
                            },
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter {
                                    send_message();
                                }
                            },
                        }
                        Button {
                            onclick: move |_| send_message(),
                            disabled: assistant_responding(),
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "20",
                                height: "20",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path {
                                    d: "M22 2 11 13"
                                }
                                path {
                                    d: "M22 2 15 22 11 13 2 9Z"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum ChatUser {
    Assistant,
    User,
}

impl ChatUser {
    fn bubble_style(&self) -> &'static str {
        match self {
            ChatUser::Assistant => "background: var(--primary-color-3); color: var(--secondary-color-4);",
            ChatUser::User => "background: var(--focused-border-color); color: var(--primary-color);",
        }
    }

    fn avatar_fallback(&self) -> &'static str {
        match self {
            ChatUser::Assistant => "AI",
            ChatUser::User => "U",
        }
    }

    fn token_style(&self) -> &'static str {
        match self {
            ChatUser::Assistant => "color: var(--secondary-color-5);",
            ChatUser::User => "color: var(--secondary-color-5);",
        }
    }
}

#[derive(PartialEq, Clone)]
struct MessageState {
    user: ChatUser,
    text: String,
    response_time: Option<Duration>,
    tokens: usize,
}

#[component]
fn Message(message: ReadSignal<MessageState>) -> Element {
    let assistant_placeholder = use_memo(move || {
        let message = message.read();
        message.user == ChatUser::Assistant && message.text.is_empty()
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
    let is_assistant = user == ChatUser::Assistant;

    let row_style = if is_assistant {
        "display: flex; gap: 0.75rem; flex-direction: row; margin-bottom: 1rem;"
    } else {
        "display: flex; gap: 0.75rem; flex-direction: row-reverse; margin-bottom: 1rem;"
    };

    let token_style = if is_assistant {
        format!("{} text-align: left;", user.token_style())
    } else {
        format!("{} text-align: right;", user.token_style())
    };

    rsx! {
        div {
            style: "{row_style}",

            // Avatar
            Avatar {
                size: AvatarImageSize::Small,
                style: "flex-shrink: 0;",
                AvatarFallback {
                    {user.avatar_fallback()}
                }
            }

            // Message bubble
            div {
                style: "max-width: 70%; display: flex; flex-direction: column;",
                div {
                    style: "padding: 0.75rem 1rem; border-radius: 1rem; {user.bubble_style()}",
                    if assistant_placeholder() {
                        div {
                            style: "display: flex; align-items: center; gap: 0.5rem; color: var(--secondary-color-6);",
                            div {
                                style: "width: 0.5rem; height: 0.5rem; background: currentColor; border-radius: 50%; animation: pulse 2s infinite;",
                            }
                            "Thinking..."
                        }
                    } else {
                        div {
                            dangerous_inner_html: "{contents}"
                        }
                    }
                }
                if let Some(tokens_per_second) = tokens_per_second() {
                    div {
                        style: "font-size: 0.75rem; margin-top: 0.25rem; padding: 0 0.25rem; {token_style}",
                        "{tokens_per_second:.1} tokens/s"
                    }
                }
            }
        }
    }
}
