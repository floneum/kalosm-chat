#![recursion_limit = "256"]
#![allow(non_snake_case)]

mod components;

use std::time::Duration;
use wasm_timer::Instant;

use components::avatar::{Avatar, AvatarFallback, AvatarImage, AvatarImageSize};
use components::button::{Button, ButtonVariant};
use components::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use components::input::Input;
use components::label::Label;
use components::progress::{Progress, ProgressIndicator};
use components::scroll_area::ScrollArea;
use components::textarea::Textarea;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapterBuilder, ExtensionOptions,
    Plugins, RenderOptions,
};
use dioxus::document::eval;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use kalosm_llama::prelude::*;

const STREAM_RENDER_INTERVAL: Duration = Duration::from_millis(33);
const CUSTOM_MODEL_PRESET_ID: &str = "custom";
const DEFAULT_MODEL_PRESET_ID: &str = "qwen-2-5-0-5b-instruct";
const PRESET_ROUTE_USER: &str = "preset";
const PRESET_ROUTE_FILE: &str = "_";

#[derive(Clone, Copy)]
struct ModelPreset {
    id: &'static str,
    family: &'static str,
    label: &'static str,
    description: &'static str,
    assistant_description: &'static str,
    supports_system_prompt: bool,
    source: fn() -> LlamaSource,
}

static MODEL_PRESETS: &[ModelPreset] = &[
    ModelPreset {
        id: "qwen-2-5-0-5b-instruct",
        family: "Qwen",
        label: "Qwen2.5 0.5B Instruct",
        description: "Small default chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_2_5_0_5b_instruct,
    },
    ModelPreset {
        id: "qwen-2-5-1-5b-instruct",
        family: "Qwen",
        label: "Qwen2.5 1.5B Instruct",
        description: "Compact Qwen2.5 chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_2_5_1_5b_instruct,
    },
    ModelPreset {
        id: "qwen-2-5-3b-instruct",
        family: "Qwen",
        label: "Qwen2.5 3B Instruct",
        description: "Balanced small Qwen2.5 chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_2_5_3b_instruct,
    },
    ModelPreset {
        id: "qwen-2-5-7b-instruct",
        family: "Qwen",
        label: "Qwen2.5 7B Instruct",
        description: "Larger Qwen2.5 chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_2_5_7b_instruct,
    },
    ModelPreset {
        id: "qwen-3-0-6b-instruct",
        family: "Qwen",
        label: "Qwen3 0.6B Instruct",
        description: "Small Qwen3 reasoning-capable chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_3_0_6b_instruct,
    },
    ModelPreset {
        id: "qwen-3-1-7b-instruct",
        family: "Qwen",
        label: "Qwen3 1.7B Instruct",
        description: "Compact Qwen3 reasoning-capable chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_3_1_7b_instruct,
    },
    ModelPreset {
        id: "qwen-3-4b-instruct",
        family: "Qwen",
        label: "Qwen3 4B Instruct",
        description: "Mid-size Qwen3 reasoning-capable chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_3_4b_instruct,
    },
    ModelPreset {
        id: "qwen-3-8b-instruct",
        family: "Qwen",
        label: "Qwen3 8B Instruct",
        description: "Larger Qwen3 reasoning-capable chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_3_8b_instruct,
    },
    ModelPreset {
        id: "qwen-3-14b-instruct",
        family: "Qwen",
        label: "Qwen3 14B Instruct",
        description: "High-capacity Qwen3 chat model.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_3_14b_instruct,
    },
    ModelPreset {
        id: "qwen-3-32b-instruct",
        family: "Qwen",
        label: "Qwen3 32B Instruct",
        description: "Largest Qwen3 text preset.",
        assistant_description:
            "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::qwen_3_32b_instruct,
    },
    ModelPreset {
        id: "llama-3-2-1b-chat",
        family: "Llama",
        label: "Llama 3.2 1B Instruct",
        description: "Small Llama 3.2 chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_3_2_1b_chat,
    },
    ModelPreset {
        id: "llama-3-2-3b-chat",
        family: "Llama",
        label: "Llama 3.2 3B Instruct",
        description: "Compact Llama 3.2 chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_3_2_3b_chat,
    },
    ModelPreset {
        id: "llama-3-1-8b-chat",
        family: "Llama",
        label: "Llama 3.1 8B Instruct",
        description: "Kalosm's default LlamaSource preset.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_3_1_8b_chat,
    },
    ModelPreset {
        id: "llama-8b-chat",
        family: "Llama",
        label: "Llama 3 8B Instruct Q5",
        description: "Llama 3 8B chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_8b_chat,
    },
    ModelPreset {
        id: "llama-8b-chat-q8",
        family: "Llama",
        label: "Llama 3 8B Instruct Q8",
        description: "Higher-precision Llama 3 8B chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_8b_chat_q8,
    },
    ModelPreset {
        id: "llama-7b-chat",
        family: "Llama",
        label: "Llama 2 7B Chat",
        description: "Llama 2 7B chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_7b_chat,
    },
    ModelPreset {
        id: "llama-13b-chat",
        family: "Llama",
        label: "Llama 2 13B Chat",
        description: "Llama 2 13B chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_13b_chat,
    },
    ModelPreset {
        id: "llama-70b-chat",
        family: "Llama",
        label: "Llama 2 70B Chat",
        description: "Large Llama 2 chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::llama_70b_chat,
    },
    ModelPreset {
        id: "tiny-llama-1-1b-chat",
        family: "Llama",
        label: "TinyLlama 1.1B Chat",
        description: "Very small Llama-compatible chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::tiny_llama_1_1b_chat,
    },
    ModelPreset {
        id: "phi-3-mini-4k-instruct",
        family: "Phi",
        label: "Phi-3 Mini 4K Instruct",
        description: "Small Microsoft Phi chat model.",
        assistant_description:
            "You are Phi, a language model from Microsoft. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::phi_3_mini_4k_instruct,
    },
    ModelPreset {
        id: "phi-3-1-mini-4k-instruct",
        family: "Phi",
        label: "Phi-3.1 Mini 4K Instruct",
        description: "Updated Phi-3 mini chat model.",
        assistant_description:
            "You are Phi, a language model from Microsoft. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::phi_3_1_mini_4k_instruct,
    },
    ModelPreset {
        id: "phi-3-5-mini-4k-instruct",
        family: "Phi",
        label: "Phi-3.5 Mini Instruct",
        description: "Phi-3.5 mini chat model.",
        assistant_description:
            "You are Phi, a language model from Microsoft. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::phi_3_5_mini_4k_instruct,
    },
    ModelPreset {
        id: "phi-4",
        family: "Phi",
        label: "Phi-4 14B",
        description: "Larger Microsoft Phi chat model.",
        assistant_description:
            "You are Phi, a language model from Microsoft. You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::phi_4,
    },
    ModelPreset {
        id: "gemma-3-270m-chat",
        family: "Gemma",
        label: "Gemma 3 270M Instruct",
        description: "Very small Gemma 3 instruction model.",
        assistant_description: "",
        supports_system_prompt: false,
        source: LlamaSource::gemma_3_270m_chat,
    },
    ModelPreset {
        id: "gemma-3-1b-chat",
        family: "Gemma",
        label: "Gemma 3 1B Instruct",
        description: "Small Gemma 3 instruction model.",
        assistant_description: "",
        supports_system_prompt: false,
        source: LlamaSource::gemma_3_1b_chat,
    },
    ModelPreset {
        id: "gemma-3-4b-chat",
        family: "Gemma",
        label: "Gemma 3 4B Instruct",
        description: "Mid-size Gemma 3 instruction model.",
        assistant_description: "",
        supports_system_prompt: false,
        source: LlamaSource::gemma_3_4b_chat,
    },
    ModelPreset {
        id: "gemma-3-12b-chat",
        family: "Gemma",
        label: "Gemma 3 12B Instruct",
        description: "Large Gemma 3 instruction model.",
        assistant_description: "",
        supports_system_prompt: false,
        source: LlamaSource::gemma_3_12b_chat,
    },
    ModelPreset {
        id: "gemma-3-27b-chat",
        family: "Gemma",
        label: "Gemma 3 27B Instruct",
        description: "Largest Gemma 3 text preset.",
        assistant_description: "",
        supports_system_prompt: false,
        source: LlamaSource::gemma_3_27b_chat,
    },
    ModelPreset {
        id: "deepseek-r1-distill-qwen-1-5b",
        family: "DeepSeek",
        label: "R1 Distill Qwen 1.5B",
        description: "Small reasoning-oriented distilled model.",
        assistant_description: "You are a reasoning assistant. Provide clear, concise answers.",
        supports_system_prompt: true,
        source: LlamaSource::deepseek_r1_distill_qwen_1_5b,
    },
    ModelPreset {
        id: "deepseek-r1-distill-qwen-7b",
        family: "DeepSeek",
        label: "R1 Distill Qwen 7B",
        description: "Qwen-based distilled reasoning model.",
        assistant_description: "You are a reasoning assistant. Provide clear, concise answers.",
        supports_system_prompt: true,
        source: LlamaSource::deepseek_r1_distill_qwen_7b,
    },
    ModelPreset {
        id: "deepseek-r1-distill-qwen-14b",
        family: "DeepSeek",
        label: "R1 Distill Qwen 14B",
        description: "Larger Qwen-based distilled reasoning model.",
        assistant_description: "You are a reasoning assistant. Provide clear, concise answers.",
        supports_system_prompt: true,
        source: LlamaSource::deepseek_r1_distill_qwen_14b,
    },
    ModelPreset {
        id: "deepseek-r1-distill-llama-8b",
        family: "DeepSeek",
        label: "R1 Distill Llama 8B",
        description: "Llama-based distilled reasoning model.",
        assistant_description: "You are a reasoning assistant. Provide clear, concise answers.",
        supports_system_prompt: true,
        source: LlamaSource::deepseek_r1_distill_llama_8b,
    },
    ModelPreset {
        id: "mistral-7b-instruct",
        family: "Mistral",
        label: "Mistral 7B Instruct v0.1",
        description: "Mistral 7B instruction model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::mistral_7b_instruct,
    },
    ModelPreset {
        id: "mistral-7b-instruct-2",
        family: "Mistral",
        label: "Mistral 7B Instruct v0.2",
        description: "Updated Mistral 7B instruction model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::mistral_7b_instruct_2,
    },
    ModelPreset {
        id: "neural-hermes-2-5-mistral-7b",
        family: "Mistral",
        label: "NeuralHermes 2.5 Mistral 7B",
        description: "Mistral-based conversational model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::neural_hermes_2_5_mistral_7b,
    },
    ModelPreset {
        id: "neural-chat-7b-v3-3",
        family: "Mistral",
        label: "Neural Chat 7B v3.3",
        description: "Intel neural-chat preset.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::neural_chat_7b_v3_3,
    },
    ModelPreset {
        id: "zephyr-7b-alpha",
        family: "Mistral",
        label: "Zephyr 7B Alpha",
        description: "Zephyr chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::zephyr_7b_alpha,
    },
    ModelPreset {
        id: "zephyr-7b-beta",
        family: "Mistral",
        label: "Zephyr 7B Beta",
        description: "Updated Zephyr chat model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::zephyr_7b_beta,
    },
    ModelPreset {
        id: "open-chat-7b",
        family: "Mistral",
        label: "OpenChat 3.5 7B",
        description: "OpenChat 3.5 conversational model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::open_chat_7b,
    },
    ModelPreset {
        id: "starling-7b-alpha",
        family: "Mistral",
        label: "Starling 7B Alpha",
        description: "Starling conversational model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::starling_7b_alpha,
    },
    ModelPreset {
        id: "starling-7b-beta",
        family: "Mistral",
        label: "Starling 7B Beta",
        description: "Updated Starling conversational model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::starling_7b_beta,
    },
    ModelPreset {
        id: "wizard-lm-7b-v2",
        family: "Mistral",
        label: "WizardLM 2 7B",
        description: "WizardLM instruction model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::wizard_lm_7b_v2,
    },
    ModelPreset {
        id: "solar-10-7b-instruct",
        family: "SOLAR",
        label: "SOLAR 10.7B Instruct",
        description: "SOLAR instruction model.",
        assistant_description: "You are a helpful assistant.",
        supports_system_prompt: true,
        source: LlamaSource::solar_10_7b_instruct,
    },
    ModelPreset {
        id: "codestral-22b",
        family: "Code",
        label: "Codestral 22B",
        description: "Code-focused Mistral model.",
        assistant_description:
            "You are a coding assistant. Provide practical, correct code and concise explanations.",
        supports_system_prompt: true,
        source: LlamaSource::codestral_22b,
    },
    ModelPreset {
        id: "llama-7b-code",
        family: "Code",
        label: "CodeLlama 7B",
        description: "Code-focused Llama model.",
        assistant_description:
            "You are a coding assistant. Provide practical, correct code and concise explanations.",
        supports_system_prompt: true,
        source: LlamaSource::llama_7b_code,
    },
    ModelPreset {
        id: "llama-13b-code",
        family: "Code",
        label: "CodeLlama 13B",
        description: "Larger CodeLlama model.",
        assistant_description:
            "You are a coding assistant. Provide practical, correct code and concise explanations.",
        supports_system_prompt: true,
        source: LlamaSource::llama_13b_code,
    },
    ModelPreset {
        id: "llama-34b-code",
        family: "Code",
        label: "CodeLlama 34B",
        description: "Largest CodeLlama preset.",
        assistant_description:
            "You are a coding assistant. Provide practical, correct code and concise explanations.",
        supports_system_prompt: true,
        source: LlamaSource::llama_34b_code,
    },
];

fn find_model_preset(id: &str) -> Option<ModelPreset> {
    MODEL_PRESETS.iter().copied().find(|preset| preset.id == id)
}

fn model_source_from_route(user: &str, model_id: &str, file: &str) -> Result<LlamaSource, String> {
    if user == PRESET_ROUTE_USER {
        let preset = find_model_preset(model_id)
            .ok_or_else(|| format!("Unknown model preset: {model_id}"))?;
        Ok((preset.source)())
    } else {
        Ok(LlamaSource::new(FileSource::huggingface(
            format!("{user}/{model_id}"),
            "main",
            file,
        )))
    }
}

fn model_supports_system_prompt(user: &str, model_id: &str) -> bool {
    user != PRESET_ROUTE_USER
        || find_model_preset(model_id)
            .map(|preset| preset.supports_system_prompt)
            .unwrap_or(true)
}

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
    let mut selected_preset = use_signal(|| DEFAULT_MODEL_PRESET_ID.to_string());
    let mut user = use_signal(|| "Qwen".to_string());
    let mut model_id = use_signal(|| "Qwen2.5-0.5B-Instruct-GGUF".to_string());
    let mut file = use_signal(|| "qwen2.5-0.5b-instruct-q4_k_m.gguf".to_string());
    let mut assistant_description = use_signal(|| {
        "You are Qwen, created by Alibaba Cloud. You are a helpful assistant.".to_string()
    });
    let mut disabled = use_signal(|| false);
    _ = use_resource(move || async move {
        if selected_preset() != CUSTOM_MODEL_PRESET_ID {
            disabled.set(false);
            return;
        }

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
        let selected_preset = selected_preset();
        let (user, model_id, file) = if selected_preset == CUSTOM_MODEL_PRESET_ID {
            (user(), model_id(), file())
        } else {
            (
                PRESET_ROUTE_USER.to_string(),
                selected_preset,
                PRESET_ROUTE_FILE.to_string(),
            )
        };
        navigator.push(Route::Home {
            assistant_description: assistant_description(),
            user,
            model_id,
            file,
        });
    };

    let selected_preset_info = find_model_preset(&selected_preset());
    let supports_system_prompt = selected_preset() == CUSTOM_MODEL_PRESET_ID
        || selected_preset_info
            .map(|preset| preset.supports_system_prompt)
            .unwrap_or(true);

    rsx! {
        div {
            style: "display: flex; align-items: center; justify-content: center; min-height: 100vh;",
            Card {
                style: "width: 100%; max-width: 34rem;",
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
                                html_for: "model-preset",
                                "Model Preset"
                            }
                            select {
                                id: "model-preset",
                                value: "{selected_preset}",
                                style: "height: 2.5rem; width: 100%; border-radius: 0.375rem; border: 1px solid var(--primary-color-6); background: var(--primary-color); color: var(--secondary-color-4); padding: 0 0.75rem; font-size: 0.875rem;",
                                onchange: move |event: FormEvent| {
                                    let preset_id = event.value();
                                    selected_preset.set(preset_id.clone());
                                    if let Some(preset) = find_model_preset(&preset_id) {
                                        assistant_description.set(preset.assistant_description.to_string());
                                    }
                                },
                                option {
                                    value: CUSTOM_MODEL_PRESET_ID,
                                    "Custom Hugging Face model"
                                }
                                for preset in MODEL_PRESETS {
                                    option {
                                        key: "{preset.id}",
                                        value: "{preset.id}",
                                        "{preset.family}: {preset.label}"
                                    }
                                }
                            }
                            if let Some(preset) = selected_preset_info {
                                p {
                                    style: "margin: 0; color: var(--secondary-color-5); font-size: 0.8125rem; line-height: 1.25rem;",
                                    "{preset.description}"
                                }
                            }
                        }

                        if selected_preset() == CUSTOM_MODEL_PRESET_ID {
                            div {
                                style: "display: flex; flex-direction: column; gap: 0.5rem;",
                                Label {
                                    html_for: "hf-user",
                                    "Hugging Face User"
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
                                disabled: !supports_system_prompt,
                                oninput: move |event: FormEvent| assistant_description.set(event.value()),
                                onkeydown: move |event: KeyboardEvent| {
                                    if event.key() == Key::Enter && !event.modifiers().shift() {
                                        start_chat();
                                    }
                                },
                            }
                            if !supports_system_prompt {
                                p {
                                    style: "margin: 0; color: var(--secondary-color-5); font-size: 0.8125rem; line-height: 1.25rem;",
                                    "This preset does not use a system prompt."
                                }
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
    let mut next_message_id = use_signal(|| 0_u64);
    let mut assistant_responding = use_signal(|| false);
    let model = use_resource(move || async move {
        let user = user();
        let model_id = model_id();
        let file = file();
        let source = model_source_from_route(&user, &model_id, &file)?;

        Llama::builder()
            .with_source(source)
            .build_with_loading_handler(move |progress| {
                loading_progress.loading_progress.set(progress.progress())
            })
            .await
            .map_err(|err| err.to_string())
    })
    .suspend()?;

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

    let start_generation = move |assistant_message_id: u64, request: GenerationRequest| {
        let model_result = {
            let read = model.read();
            match &*read {
                Ok(model) => Ok(model.clone()),
                Err(error) => Err(error.clone()),
            }
        };
        let route_user = user();
        let route_model_id = model_id();
        let assistant_description = assistant_description();

        spawn(async move {
            match model_result {
                Ok(model) => {
                    let mut chat = model.chat();
                    if model_supports_system_prompt(&route_user, &route_model_id)
                        && !assistant_description.is_empty()
                    {
                        chat = chat.with_system_prompt(assistant_description);
                    }
                    chat = chat.with_messages(request.history);

                    let mut stream = chat
                        .add_message(ChatMessage::new(MessageType::UserMessage, request.prompt));
                    let mut pending_text = String::new();
                    let mut pending_tokens = 0usize;
                    let mut last_render = Instant::now();
                    let mut decode_start = None;
                    let mut throughput_tokens = 0usize;
                    while let Some(new_text) = stream.next().await {
                        if decode_start.is_some() {
                            throughput_tokens += 1;
                        } else {
                            decode_start = Some(Instant::now());
                        }
                        pending_text.push_str(&new_text);
                        pending_tokens += 1;
                        if last_render.elapsed() >= STREAM_RENDER_INTERVAL {
                            flush_pending_message(
                                &mut messages,
                                assistant_message_id,
                                &mut pending_text,
                                &mut pending_tokens,
                            );
                            last_render = Instant::now();
                        }
                    }
                    flush_pending_message(
                        &mut messages,
                        assistant_message_id,
                        &mut pending_text,
                        &mut pending_tokens,
                    );
                    finish_message_generation(
                        &mut messages,
                        assistant_message_id,
                        decode_start.map(|start| start.elapsed()),
                        throughput_tokens,
                    );
                }
                Err(error) => {
                    update_message_error(
                        &mut messages,
                        assistant_message_id,
                        format!("Error: {error}"),
                    );
                }
            }
            assistant_responding.set(false);
        });
    };

    let mut send_message = move || {
        if assistant_responding() {
            return;
        }
        let current_msg = current_message.take();
        if current_msg.trim().is_empty() {
            return;
        }

        let user_message_id = next_message_id();
        next_message_id += 1;
        let assistant_message_id = next_message_id();
        next_message_id += 1;

        let request = {
            let mut messages_mut = messages.write();
            messages_mut.push(MessageState::new(
                user_message_id,
                ChatUser::User,
                current_msg,
                false,
            ));
            let request = generation_request_from_messages(&messages_mut);
            messages_mut.push(MessageState::new(
                assistant_message_id,
                ChatUser::Assistant,
                String::new(),
                true,
            ));
            request
        };

        if let Some(request) = request {
            assistant_responding.set(true);
            start_generation(assistant_message_id, request);
        }
    };

    let mut save_message = move |(message_id, text): (u64, String)| {
        if assistant_responding() {
            return;
        }

        if let Some(message) = messages
            .write()
            .iter_mut()
            .find(|message| message.id == message_id)
        {
            message.text = text;
            message.streaming = false;
            message.response_time = None;
            message.tokens = 0;
            message.throughput_tokens = 0;
        }
    };

    let mut delete_message = move |message_id: u64| {
        if assistant_responding() {
            return;
        }

        messages.write().retain(|message| message.id != message_id);
    };

    let mut regenerate_message = move |message_id: u64| {
        if assistant_responding() {
            return;
        }

        let plan = {
            let messages_read = messages.read();
            let Some(index) = messages_read
                .iter()
                .position(|message| message.id == message_id)
            else {
                return;
            };
            let truncate_len = match messages_read[index].user {
                ChatUser::User => index + 1,
                ChatUser::Assistant => index,
            };
            let request = generation_request_from_messages(&messages_read[..truncate_len]);
            request.map(|request| (truncate_len, request))
        };

        let Some((truncate_len, request)) = plan else {
            return;
        };

        let assistant_message_id = next_message_id();
        next_message_id += 1;
        {
            let mut messages_mut = messages.write();
            messages_mut.truncate(truncate_len);
            messages_mut.push(MessageState::new(
                assistant_message_id,
                ChatUser::Assistant,
                String::new(),
                true,
            ));
        }

        assistant_responding.set(true);
        start_generation(assistant_message_id, request);
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
                            key: "{message.id}",
                            message,
                            disabled: assistant_responding(),
                            onsave: move |payload| save_message(payload),
                            ondelete: move |message_id| delete_message(message_id),
                            onregenerate: move |message_id| regenerate_message(message_id),
                        }
                    }
                }
            }

            // Input area
            Card {
                CardContent {
                    style: "padding: 0rem 1rem;",
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

fn flush_pending_message(
    messages: &mut Signal<Vec<MessageState>>,
    message_id: u64,
    pending_text: &mut String,
    pending_tokens: &mut usize,
) {
    if pending_text.is_empty() && *pending_tokens == 0 {
        return;
    }

    let mut messages = messages.write();
    let Some(message) = messages.iter_mut().find(|message| message.id == message_id) else {
        pending_text.clear();
        *pending_tokens = 0;
        return;
    };

    message.text.push_str(pending_text);
    message.tokens += *pending_tokens;
    pending_text.clear();
    *pending_tokens = 0;
}

fn finish_message_generation(
    messages: &mut Signal<Vec<MessageState>>,
    message_id: u64,
    response_time: Option<Duration>,
    throughput_tokens: usize,
) {
    let mut messages = messages.write();
    let Some(message) = messages.iter_mut().find(|message| message.id == message_id) else {
        return;
    };

    message.response_time = response_time;
    message.throughput_tokens = throughput_tokens;
    message.streaming = false;
}

fn update_message_error(messages: &mut Signal<Vec<MessageState>>, message_id: u64, error: String) {
    let mut messages = messages.write();
    let Some(message) = messages.iter_mut().find(|message| message.id == message_id) else {
        return;
    };

    message.text = error;
    message.streaming = false;
}

#[derive(PartialEq, Clone, Copy)]
enum ChatUser {
    Assistant,
    User,
}

impl ChatUser {
    fn bubble_style(&self) -> &'static str {
        match self {
            ChatUser::Assistant => {
                "background: var(--primary-color-3); color: var(--secondary-color-4);"
            }
            ChatUser::User => {
                "background: var(--focused-border-color); color: var(--primary-color);"
            }
        }
    }

    fn avatar_fallback(&self) -> &'static str {
        match self {
            ChatUser::Assistant => "AI",
            ChatUser::User => "U",
        }
    }

    fn message_type(&self) -> MessageType {
        match self {
            ChatUser::Assistant => MessageType::ModelAnswer,
            ChatUser::User => MessageType::UserMessage,
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
    id: u64,
    user: ChatUser,
    text: String,
    response_time: Option<Duration>,
    tokens: usize,
    throughput_tokens: usize,
    streaming: bool,
}

impl MessageState {
    fn new(id: u64, user: ChatUser, text: String, streaming: bool) -> Self {
        Self {
            id,
            user,
            text,
            response_time: None,
            tokens: 0,
            throughput_tokens: 0,
            streaming,
        }
    }
}

#[derive(Clone)]
struct GenerationRequest {
    history: Vec<ChatMessage>,
    prompt: String,
}

fn generation_request_from_messages(messages: &[MessageState]) -> Option<GenerationRequest> {
    let user_index = messages.iter().rposition(|message| {
        message.user == ChatUser::User && !message.streaming && !message.text.trim().is_empty()
    })?;
    let history = messages[..user_index]
        .iter()
        .filter_map(message_to_chat_message)
        .collect();

    Some(GenerationRequest {
        history,
        prompt: messages[user_index].text.clone(),
    })
}

fn message_to_chat_message(message: &MessageState) -> Option<ChatMessage> {
    if message.streaming || message.text.trim().is_empty() {
        return None;
    }

    Some(ChatMessage::new(
        message.user.message_type(),
        message.text.clone(),
    ))
}

fn render_markdown(text: &str) -> String {
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
}

#[component]
fn Message(
    message: MessageState,
    disabled: bool,
    onsave: EventHandler<(u64, String)>,
    ondelete: EventHandler<u64>,
    onregenerate: EventHandler<u64>,
) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut draft = use_signal(String::new);

    let user = message.user;
    let is_assistant = user == ChatUser::Assistant;
    let assistant_placeholder = is_assistant && message.text.is_empty();
    let contents = (!is_assistant || !message.streaming).then(|| render_markdown(&message.text));
    let tokens_per_second = message.response_time.and_then(|response_time| {
        let seconds = response_time.as_secs_f64();
        (message.throughput_tokens > 0 && seconds > 0.0)
            .then(|| message.throughput_tokens as f64 / seconds)
    });

    let row_style = if is_assistant {
        "display: flex; gap: 0.75rem; flex-direction: row; margin-bottom: 0.5rem;"
    } else {
        "display: flex; gap: 0.75rem; flex-direction: row-reverse; margin-bottom: 0.5rem;"
    };

    let token_style = if is_assistant {
        format!("{} text-align: left;", user.token_style())
    } else {
        format!("{} text-align: right;", user.token_style())
    };

    let message_id = message.id;
    let edit_text = message.text.clone();
    let escape_edit_text = edit_text.clone();
    let cancel_edit_text = edit_text.clone();
    let double_click_edit_text = edit_text.clone();
    let start_edit_text = edit_text;
    let edit_rows = if is_assistant { 8 } else { 4 };
    let action_style = if is_assistant {
        "justify-content: flex-start;"
    } else {
        "justify-content: flex-end;"
    };
    let save_disabled = disabled || draft().trim().is_empty();
    let bubble_width = if is_editing() {
        "width: min(42rem, 70vw);"
    } else {
        "max-width: 70%;"
    };

    rsx! {
        div {
            style: "{row_style}",

            // Avatar
            Avatar {
                size: AvatarImageSize::Small,
                style: "flex-shrink: 0;",
                if !is_assistant {
                    AvatarImage {
                        src: "https://avatars.githubusercontent.com/u/66571940?s=96&v=4",
                        alt: "User avatar",
                    }
                }
                AvatarFallback {
                    {user.avatar_fallback()}
                }
            }

            // Message bubble
            div {
                style: "{bubble_width} display: flex; flex-direction: column;",
                div {
                    style: "padding: 0.5rem; border-radius: 1rem; {user.bubble_style()}",
                    ondoubleclick: move |event| {
                        if !disabled && !message.streaming {
                            draft.set(double_click_edit_text.clone());
                            is_editing.set(true);
                            event.prevent_default();
                        }
                    },
                    if is_editing() {
                        Textarea {
                            value: "{draft}",
                            rows: edit_rows,
                            disabled,
                            style: "width: 100%; min-height: 7rem; color: var(--secondary-color-4); resize: vertical;",
                            oninput: move |event: FormEvent| draft.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Escape {
                                    draft.set(escape_edit_text.clone());
                                    is_editing.set(false);
                                }
                            },
                        }
                        div {
                            style: "display: flex; justify-content: flex-end; gap: 0.35rem; margin-top: 0.5rem;",
                            Button {
                                variant: ButtonVariant::Ghost,
                                style: "height: 1.75rem; width: 1.75rem; padding: 0;",
                                disabled: save_disabled,
                                title: "Save",
                                "aria-label": "Save message",
                                onclick: move |_| {
                                    onsave.call((message_id, draft()));
                                    is_editing.set(false);
                                },
                                MessageIcon { icon: MessageActionIcon::Save }
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                style: "height: 1.75rem; width: 1.75rem; padding: 0;",
                                disabled,
                                title: "Cancel",
                                "aria-label": "Cancel edit",
                                onclick: move |_| {
                                    draft.set(cancel_edit_text.clone());
                                    is_editing.set(false);
                                },
                                MessageIcon { icon: MessageActionIcon::Cancel }
                            }
                        }
                    } else {
                        if assistant_placeholder {
                            div {
                                style: "display: flex; align-items: center; gap: 0.5rem; color: var(--secondary-color-6);",
                                div {
                                    style: "width: 0.5rem; height: 0.5rem; background: currentColor; border-radius: 50%; animation: pulse 2s infinite;",
                                }
                                "Thinking..."
                            }
                        } else if is_assistant && message.streaming {
                            pre {
                                style: "margin: 0; white-space: pre-wrap; overflow-wrap: anywhere; font: inherit;",
                                "{message.text}"
                            }
                        } else if let Some(contents) = contents {
                            div {
                                dangerous_inner_html: "{contents}"
                            }
                        }
                    }
                }
                if let Some(tokens_per_second) = tokens_per_second {
                    div {
                        style: "font-size: 0.75rem; margin-top: 0.25rem; padding: 0 0.25rem; {token_style}",
                        "{tokens_per_second:.1} tokens/s"
                    }
                }
                if !message.streaming && !is_editing() {
                    div {
                        style: "display: flex; gap: 0.25rem; margin-top: 0.25rem; padding: 0 0.125rem; {action_style}",
                        Button {
                            variant: ButtonVariant::Ghost,
                            style: "height: 1.75rem; width: 1.75rem; padding: 0;",
                            disabled,
                            title: "Edit",
                            "aria-label": "Edit message",
                            onclick: move |_| {
                                draft.set(start_edit_text.clone());
                                is_editing.set(true);
                            },
                            MessageIcon { icon: MessageActionIcon::Edit }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            style: "height: 1.75rem; width: 1.75rem; padding: 0;",
                            disabled,
                            title: "Regenerate",
                            "aria-label": "Regenerate from this message",
                            onclick: move |_| onregenerate.call(message_id),
                            MessageIcon { icon: MessageActionIcon::Regenerate }
                        }
                        Button {
                            variant: ButtonVariant::Ghost,
                            style: "height: 1.75rem; width: 1.75rem; padding: 0; color: var(--destructive-color, #dc2626);",
                            disabled,
                            title: "Delete",
                            "aria-label": "Delete message",
                            onclick: move |_| ondelete.call(message_id),
                            MessageIcon { icon: MessageActionIcon::Delete }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum MessageActionIcon {
    Edit,
    Regenerate,
    Delete,
    Save,
    Cancel,
}

#[component]
fn MessageIcon(icon: MessageActionIcon) -> Element {
    let common_style = "display: block;";

    match icon {
        MessageActionIcon::Edit => rsx! {
            svg {
                style: "{common_style}",
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M12 20h9" }
                path { d: "M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" }
            }
        },
        MessageActionIcon::Regenerate => rsx! {
            svg {
                style: "{common_style}",
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
                path { d: "M3 21v-5h5" }
                path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
                path { d: "M16 8h5V3" }
            }
        },
        MessageActionIcon::Delete => rsx! {
            svg {
                style: "{common_style}",
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M3 6h18" }
                path { d: "M8 6V4h8v2" }
                path { d: "M19 6l-1 14H6L5 6" }
                path { d: "M10 11v6" }
                path { d: "M14 11v6" }
            }
        },
        MessageActionIcon::Save => rsx! {
            svg {
                style: "{common_style}",
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M20 6 9 17l-5-5" }
            }
        },
        MessageActionIcon::Cancel => rsx! {
            svg {
                style: "{common_style}",
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M18 6 6 18" }
                path { d: "m6 6 12 12" }
            }
        },
    }
}
