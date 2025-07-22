use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
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
    rsx! {
        div {
            class: "text-3xl font-bold underline",
            "HotDog!"
          }

    }
}
