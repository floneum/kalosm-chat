use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image {
        data: String, // base64 encoded
        caption: Option<String>,
        analysis: Option<String>, // AI analysis result
    },
    Audio {
        data: String, // base64 encoded
        duration: Option<f32>,
        transcription: Option<String>,
    },
    File {
        name: String,
        data: String, // base64 encoded
        file_type: String,
        processed_content: Option<String>, // Extracted/processed content
    },
    GeneratedImage {
        data: String, // base64 encoded
        prompt: String,
        model_used: String,
    },
    WebScraping {
        url: String,
        title: Option<String>,
        content: String,
        summary: Option<String>,
    },
    StructuredData {
        data: String, // JSON string
        schema: Option<String>,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: MessageContent,
    pub is_user: bool,
    pub tokens_per_second: Option<f32>,
    pub timestamp: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct MessageProps {
    chat: ChatMessage,
    #[props(default = false)]
    is_typing: bool,
}

#[component]
pub fn Message(props: MessageProps) -> Element {
    let is_user = props.chat.is_user;
    let bubble_classes = if is_user {
        "rounded-xl px-4 py-2 shadow-sm text-sm break-words whitespace-pre-wrap bg-indigo-500 text-white ml-auto min-w-[80px] max-w-full"
    } else {
        "rounded-xl px-4 py-2 shadow-sm text-sm break-words whitespace-pre-wrap bg-zinc-600 text-gray-100 mr-auto min-w-[80px] max-w-full"
    };

    rsx! {
        div {
            class: "flex items-end space-x-2 mb-4 max-w-full",
            class: if is_user { "justify-end" } else { "justify-start" },

            // Avatar (trailing or leading based on user)
            if !is_user {
                Avatar {
                    is_user: false,
                    class: Some("ring-2 bg-zinc-600 ring-gray-400".into()),
                    style: Some("box-shadow: 0 0 6px rgba(0, 0, 0, 0.5);".into()),
                    label: Some(rsx!(
                        img {
                            class: "w-full h-full object-cover rounded-full",
                            src: "https://thumbs.wbm.im/pw/small/bae0a5cb7bbf94c7cbd8e7413c20d826.png",
                        }
                    ))
                }
            }

            // Message bubble + timestamp
            div {
                class: "flex flex-col max-w-[75%] space-y-1",
                class: if is_user { "items-end" } else { "items-start" },

                // Username and optional timestamp
                div {
                    class: "flex items-center space-x-2",
                    span {
                        class: "text-xs font-semibold text-gray-400",
                        if is_user { "You" } else { "Bot" }
                    },
                    span {
                        class: "text-xs text-gray-500",
                        "{props.chat.timestamp}"
                    }
                }

                // Message bubble content
                div {
                    class: bubble_classes,

                    // Check if this is a typing indicator
                    if props.is_typing {
                        MsgBubbles {}
                    } else {
                        match &props.chat.content {
                            MessageContent::Text(text) => rsx! {
                                div { "{text}" }
                            },
                            MessageContent::Error(error) => rsx! {
                                div {
                                    class: "text-red-400 flex items-center space-x-2",
                                    span { "⚠️" }
                                    span { "{error}" }
                                }
                            },
                            MessageContent::Image { data, caption, analysis } => rsx! {
                                div {
                                    class: "space-y-2",
                                    img {
                                        class: "max-w-full h-auto object-cover rounded-lg",
                                        src: "data:image/png;base64,{data}"
                                    }
                                    if let Some(cap) = caption {
                                        div { class: "text-sm italic", "{cap}" }
                                    }
                                    if let Some(anal) = analysis {
                                        div { class: "text-sm text-blue-300", "Analysis: {anal}" }
                                    }
                                }
                            },
                            MessageContent::Audio { transcription, duration, .. } => rsx! {
                                div {
                                    class: "space-y-2",
                                    div {
                                        class: "bg-gray-700 p-2 rounded text-sm flex items-center space-x-2",
                                        span { "🎵 Audio" }
                                        if let Some(dur) = duration {
                                            span { class: "text-xs text-gray-400", "{dur:.1}s" }
                                        }
                                    }
                                    if let Some(trans) = transcription {
                                        div { class: "text-sm", "📝 {trans}" }
                                    }
                                }
                            },
                            _ => rsx! {
                                div {
                                    class: "text-gray-400 italic",
                                    "Unsupported content type (coming soon)"
                                }
                            }
                        }
                    }

                    // Token speed (optional)
                    if let Some(tps) = props.chat.tokens_per_second {
                        if !is_user {
                            div {
                                class: "text-xs mt-1 text-gray-400",
                                "{tps:.1} tokens/s"
                            }
                        }
                    }
                }
            }

            // Trailing Avatar if user
            if is_user {
                Avatar {
                    is_user: true,
                    class: Some("ring-2 ring-yellow-400".into()),
                    style: Some("box-shadow: 0 0 6px rgba(255, 200, 0, 0.5);".into()),
                    label: Some(rsx!(
                        img {
                            class: "w-full h-full object-cover rounded-full",
                            src: "https://plus.unsplash.com/premium_photo-1664474619075-644dd191935f?fm=jpg&q=60&w=3000&ixlib=rb-4.1.0&ixid=M3wxMjA3fDB8MHxzZWFyY2h8MXx8aW1hZ2V8ZW58MHx8MHx8fDA%3D",
                        }
                    ))
                }
            }
        }
    }
}

#[component]
pub fn MsgBubbles() -> Element {
    rsx! {
        div {
            class: "rounded-xl px-4 py-1  inline-block",
                                            
            div {
                class: "flex space-x-1 items-center",
                
                div {
                    class: "w-2 h-2 rounded-full typing-dot",
                    style: "background: #72767d;"
                }
                div {
                    class: "w-2 h-2 rounded-full typing-dot",
                    style: "background: #72767d;"
                }
                div {
                    class: "w-2 h-2 rounded-full typing-dot",
                    style: "background: #72767d;"
                }
            }
        }
    }
}