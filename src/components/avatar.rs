use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct AvatarProps {
    pub is_bot: bool,
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub style: Option<String>,
    #[props(optional)]
    pub label: Option<Element>,
    #[props(optional)]
    pub size: Option<String>,
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let default_style = if props.is_bot {
        "background: #40444b;"
    } else {
        "background: linear-gradient(135deg, #7289da, #5865f2);"
    };

    let default_label = if props.is_bot {
        rsx!( "🤖" )
    } else {
        rsx!( "You" )
    };

    let size_class = props.size.as_deref().unwrap_or("w-10 h-10");

    rsx! {
        div {
            class: format_args!(
                "flex-shrink-0 rounded-full overflow-hidden flex items-center justify-center {} {}",
                size_class,
                props.class.as_deref().unwrap_or("")
            ),
            style: format_args!(
                "{} {}",
                default_style,
                props.style.as_deref().unwrap_or("")
            ),

            if let Some(label) = props.label.clone() {
                {label}
            } else {
                span {
                    class: "text-white text-sm font-semibold",
                    {default_label}
                }
            }
        }
    }
}
