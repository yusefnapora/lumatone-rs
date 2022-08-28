#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this once things settle down a bit...
mod components;
mod drawing;

use components::color_wheel::ColorWheel;

use dioxus::prelude::*;

fn main() {
    dioxus::desktop::launch(app);
}

fn app(cx: Scope) -> Element {
    cx.render(rsx! (
        ColorWheel { radius: 500.0 }
    ))
}