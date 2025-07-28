
use dioxus::prelude::*;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use kalosm::language::{ Llama,};
use kalosm::language::ChatModelExt;
use crate::components::*;


#[derive(Clone, Debug, PartialEq)]
enum ProcessingState {
    Idle,
    ProcessingText,
    ProcessingImage,
    ProcessingAudio,
    ProcessingFile,
    GeneratingImage,
    ScrapingWeb,
    ExtractingData,
}
#[component]
pub fn Home() -> Element {
    let mut messages = use_signal(|| Vec::<ChatMessage>::new());
    let mut is_loading = use_signal(|| false);
    let mut show_attachment_menu = use_signal(|| false);
    let _processing_state = use_signal(|| ProcessingState::Idle);

    // Enhanced Chat - Personas and System Prompts
    let mut current_persona = use_signal(|| "buddy".to_string());


    // System prompt generator for different personas
    let get_system_prompt = move |persona: &str| -> String {
        match persona {
            "buddy" => "Hey! I'm your friendly chat buddy who loves having genuine conversations. I'm curious, supportive, and always up for a good chat about anything. I keep things real and fun while being genuinely helpful. Think of me as that friend who's always got your back!".to_string(),
            _ => "Hey! I'm your friendly chat buddy who's here to help and have great conversations with you.".to_string(),
        }
    };


    // Model loading using Kalosm 0.4 format - now works on all platforms including iOS
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    let model = use_resource(move || {
        async move {
            Llama::new_chat()
                .await
        }
    });

    let mut send_message = move |msg: String| {
        let message =  ChatMessage {
            content: MessageContent::Text(msg.clone()),
            is_user: true,
            tokens_per_second: None,
            timestamp: "now".to_string(),
        }; 
        messages.write().push(message);
        is_loading.set(true);
        
        spawn(async move {
            // We spawn up another thread for the bot response since user does not have to wait for chat to finish
            match model.read().as_ref() {
              None => {
                messages.write().push(ChatMessage {
                    content: MessageContent::Error("Model is still loading, please wait...".to_string()),
                    is_user: false,
                    tokens_per_second: None,
                    timestamp: "now".to_string(),
                });
              },
              Some(model) => {
                    match model { 
                        Ok(model_instance) => {
                            // create chat context with enhanced system prompt
                            let mut chat = {
                                 // Create new chat with persona-based system prompt
                                 let system_prompt = get_system_prompt(&current_persona());
                                 let new_chat = model_instance.chat()
                                     .with_system_prompt(system_prompt);
                                 new_chat
                            };
                            
                            // Send the message to the chat and get the response
                            // Add the response to the messages list
                            match chat.add_message(msg).await {
                               Ok(response) => {
                                   let response_text = response;

                                   // Add the actual response message
                                   messages.write().push(ChatMessage {
                                       content: MessageContent::Text(response_text),
                                       is_user: false,
                                       tokens_per_second: Some(25.0),
                                       timestamp: "now".to_string(),
                                   });
                                   // Auto-scroll is handled by the use_effect hook
                               },
                               Err(e) => {
                                    messages.write().push(ChatMessage {
                                        content: MessageContent::Error(format!("Model failed to respond - {}", e)),
                                        is_user: false,
                                        tokens_per_second: None,
                                        timestamp: "now".to_string(),
                                    });
                               }    
                            } 
                        },
                        Err(e) => {
                            messages.write().push(ChatMessage {
                                content: MessageContent::Error(format!("Model failed to load - {}", e)),
                                is_user: false,
                                tokens_per_second: None,
                                timestamp: "now".to_string(),
                            });
                        }
                    }
              },
            }
            is_loading.set(false);
        });
        

    };


    // For auto-scrolling in native mode, we'll use a simpler approach
    // Create a signal to track if we need to scroll
    let mut messages_count = use_signal(|| 0);
    
    // Update the count whenever messages change
    use_effect(move || {
        messages_count.set(messages.read().len());
    });
    
    rsx! {
        div {
            class: "flex flex-col h-screen bg-[#2A2928]",

            // Header
            div {
                class: "px-4 py-2 border-b text-white border-zinc-700  text-xl font-semibold",
                "Kalosm Chat"
            }

            // Message list - use a regular div with overflow
            div {
                id: "messages-container",
                class: "flex-1 p-4 overflow-y-auto",
                // This key forces the component to re-render when messages_count changes
                // which helps with scrolling in some implementations
                key: "{messages_count}",
                for (i, message) in messages.read().iter().enumerate() {
                    Message {
                          key: "{i}",
                        chat : message.clone()
                    }
                }
                if is_loading() {
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
            }

             div {
                    class: "px-4 py-3 border-b",
                    style: "border-color: #3f4147;",
                    
                    // Input field with integrated attachment controls
                    div {
                        class: "flex items-center space-x-2",
                        
                        // Attachment button (left side)
                        p {
                            class: "p-2 text-white rounded cursor-pointer transition-colors flex-shrink-0",
                           
                            onclick: move |_| {
                                show_attachment_menu.set(!show_attachment_menu());
                            },
                            if show_attachment_menu() { "×" } else { "+" }
                        }
                        
                        // Main input field container with integrated send button
                       TextInput {
                        placeholder: "Ask anything...".to_string(),
                        on_save: move |input| send_message(input),
                       }
                       
                    }
            }

        }
    }
}