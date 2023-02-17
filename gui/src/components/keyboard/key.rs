use dioxus::prelude::*;
use palette::LinSrgb;

use crate::drawing::{color::ToHexColorStr, hexagon_svg_points, Point, Float};

#[derive(PartialEq, Props)]
pub struct KeyProps {
  fill_color: String, // TODO: use LinSrgb?
  center: Point,
  size: Float,
  #[props(into)]
  label: Option<String>,
  label_color: Option<LinSrgb>,
}

pub fn Key(cx: Scope<KeyProps>) -> Element {
  let fill = cx.props.fill_color.clone();
  let stroke = "black"; // TODO: add to props?
  let center = cx.props.center;
  let size = cx.props.size;
  let points = hexagon_svg_points(center, size);

  let label = cx.props.label.clone().unwrap_or(String::new());
  let label_color = cx.props.label_color
    .map(|c| c.to_hex_color())
    .unwrap_or(String::from("white"));

  cx.render(rsx!{
    g {
      polygon {
        fill: "{fill}",
        stroke: stroke,
        points: "{points}",
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