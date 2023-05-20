use dioxus::prelude::*;
use palette::LinSrgb;

use crate::components::keyboard::{coords::Hex, layout::Layout};
use crate::drawing::color::ToHexColorStr;

#[derive(Props)]
pub struct KeyProps<'a> {
	layout: &'a Layout,
  fill_color: String, // TODO: use LinSrgb?
  coord: Hex,

  on_click: Option<EventHandler<'a, Hex>>,

  #[props(into)]
  label: Option<String>,
  label_color: Option<LinSrgb>,
}

pub fn Key<'a>(cx: Scope<'a, KeyProps<'a>>) -> Element {
  let fill = cx.props.fill_color.clone();
  let stroke = "black"; // TODO: add to props?
	let layout = cx.props.layout;
	let center = layout.hex_to_pixel(cx.props.coord);
	let points = layout.svg_polygon_points(cx.props.coord);

  let label = cx.props.label.clone().unwrap_or(String::new());
  let label_color = cx.props.label_color
    .map(|c| c.to_hex_color())
    .unwrap_or(String::from("white"));

  let coord = cx.props.coord;

  cx.render(rsx!{
    g {
      polygon {
        fill: "{fill}",
        stroke: stroke,
        points: "{points}",
        onclick: move |_event| {
          if let Some(handler) = &cx.props.on_click {
            handler.call(coord);
          }
        },
      }
      text {
        x: center.x,
        y: center.y,
        text_anchor: "middle",
        stroke: "{label_color}",
        fill: "{label_color}",

        label
      }
    }
  })
}
