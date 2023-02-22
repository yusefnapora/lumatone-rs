use std::time::Duration;

use dioxus::prelude::*;
use dioxus_desktop::use_eval;
use serde_json::Value::Number;



pub fn use_size_observer<'a>(cx: &'a ScopeState, element_id: String) -> &'a UseState<Option<(f64, f64)>> {
  let size_state = use_state(cx, || None);
  let eval = use_eval(cx);
  let poll_interval = Duration::new(1, 0);

  use_coroutine(cx, |mut _rx:UnboundedReceiver<()>| {
    to_owned![size_state, eval];

    let get_size_js = format!(r"
      let el = document.getElementById('{element_id}')
      return {{
        width: el?.offsetWidth, 
        height: el?.offsetHeight
      }}
    ");

    // println!("js script:\n{get_size_js}");
    async move {
      // println!("size observer for element id {element_id} starting");
      let mut first_iteration = true;
      loop {
        if first_iteration {
          first_iteration = false;
        } else {
          // periodically poll for size changes. ideally we'd be able to use
          // a ResizeObserver instead, but that would require calling a rust fn
          // from a JS callback, which I don't think is possible yet.
          // revisit this whole approach once 
          // [node refs](https://github.com/DioxusLabs/dioxus/issues/631)
          // are implemented.
          tokio::time::sleep(poll_interval).await;
        }

        match eval(get_size_js.clone()).await {
          Ok(value) => {
            // println!("Got size value from js: {:?}", value);
            match size_from_json_value(&value) {
              Ok(size) => {
                size_state.set(Some(size));
              },
              Err(err) => {
                eprintln!("error unpacking JS size value: {err}");
                continue;
              }
            }
          },
          Err(err) => {
            eprintln!("error getting size for element {element_id}: {err}");
            continue;
          }
        };
      }
    }
  });
  size_state
}


// TODO: use proper error type
fn size_from_json_value(value: &serde_json::Value) -> Result<(f64, f64), String> {
  match (&value["width"], &value["height"]) {
    (Number(w), Number(h)) => {
      let w = w.as_f64().ok_or(format!("invalid js number {:?}", w))?;
      let h = h.as_f64().ok_or(format!("invalid JS number: {:?}", h))?;
      Ok((w, h))
    },
    _ => Err(format!("Unexpected json value type: {:?}", value))
  }
}