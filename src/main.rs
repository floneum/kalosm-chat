mod components;
mod data;

use dioxus::prelude::*;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use metal::*;
use crate::components::message::{Message, ChatMessage, MessageContent};

fn main() {
    // Only run metal detection on iOS devices
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    detect_metal();
    
    launch(app);
}

#[component]
fn app() -> Element {
    rsx! {
        // The Stylesheet component inserts a style link into the head of the document
        document::Stylesheet {
            // Urls are relative to your Cargo.toml file
            href: asset!("/assets/tailwind.css")
        },
       
       Home {}
    }
}


#[component]
fn Home() -> Element {
    let messages = vec![
        ChatMessage {
            is_user: true,
            content: MessageContent::Text("Hello, bot!".to_string()),
            timestamp: "10:00 AM".to_string(),
            tokens_per_second: None,
        },
        ChatMessage {
            is_user: false,
            content: MessageContent::Text("Hello! How can I help you today? Here's some **bold** text and `code`.".to_string()),
            timestamp: "10:01 AM".to_string(),
            tokens_per_second: Some(150.3),
        },
        ChatMessage {
            is_user: true,
            content: MessageContent::Image {
                data: data::mock_image::MOCK_IMAGE.to_string(),
                caption: Some("A single pixel".to_string()),
                analysis: Some("This is a 1x1 black pixel.".to_string()),
            },
            timestamp: "10:03 AM".to_string(),
            tokens_per_second: None,
        },
        ChatMessage {
            is_user: false,
            content: MessageContent::Audio {
                data: "".to_string(),
                duration: Some(5.2),
                transcription: Some("This is a test audio message.".to_string()),
            },
            timestamp: "10:04 AM".to_string(),
            tokens_per_second: None,
        },
        ChatMessage {
            is_user: false,
            content: MessageContent::Error("I'm sorry, I couldn't process that request.".to_string()),
            timestamp: "10:05 AM".to_string(),
            tokens_per_second: None,
        },
    ];

    rsx! {
        div {
            class: "flex flex-col h-screen bg-[#2A2928]",

            // Header
            div {
                class: "p-4 border-b text-white border-zinc-700  text-xl font-semibold",
                "chat"
            }

            // Message list
            div {
                class: "flex-1 p-4 overflow-y-auto",
                for message in messages {
                    Message {

                        chat: message
                    }
                }
                // Typing indicator example
                Message {
                    chat: ChatMessage {
                        is_user: false,
                        content: MessageContent::Text("".to_string()),
                        timestamp: "".to_string(),
                        tokens_per_second: None,
                    },
                    is_typing: true
                }
            }

            // Input area (placeholder)
            div {
                class: "p-4 border-t border-zinc-700",
                div {
                    class: "bg-zinc-700 rounded-lg p-4 text-white",
                    "Message input..."
                }
            }
        }
    }
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn detect_metal() {
    println!("Detecting Metal...");
    if let Some(device) = Device::system_default() {
        println!("✅ Metal device found: {}", device.name());
    } else {
        println!("❌ Metal not available.");
    }
}