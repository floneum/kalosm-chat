mod components;
mod data;
mod views;

use dioxus::prelude::*;
use components::*;
use views::*;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use metal::*;

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




#[cfg(any(target_os = "ios", target_os = "macos"))]
fn detect_metal() {
    println!("Detecting Metal...");
    if let Some(device) = Device::system_default() {
        println!("✅ Metal device found: {}", device.name());
    } else {
        println!("❌ Metal not available.");
    }
}