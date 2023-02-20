use dioxus::prelude::*;
use crate::components::tabs::{TabContainer, TabItem};

pub fn Scratchpad(cx: Scope<()>) -> Element {
  cx.render(rsx! {
    div {
      width: "600px",
      height: "600px",
    
      TabContainer {
        tabs: vec![
          TabItem {
            title: "Foo",
            id: "foo",
            content: cx.render(rsx! {
              div { 
                "foo"
              }
            })
          },

          TabItem {
            title: "Bar",
            id: "bar",
            content: cx.render(rsx! {
              div { 
                "bar"
              }
            })
          },

          TabItem {
            title: "Baz",
            id: "baz",
            content: cx.render(rsx! {
              div { 
                "baz"
              }
            })
          }          
        ]
      }
    }
  })
}
