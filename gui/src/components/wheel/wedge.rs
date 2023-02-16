use dioxus::prelude::*;
use palette::LinSrgb;

use crate::drawing::{
  arc_svg_path, color::ToHexColorStr, line_to, polar_to_cartesian, Angle, Float, Point,
};

type Color = LinSrgb;

#[derive(PartialEq, Props)]
pub struct WedgeProps {
  radius: Float,
  center: Point,
  color: Color,
  text_color: Color,
  #[props(into)]
  label: String,
  rotation: Float,
  arc_angle: Angle,
}

/// A component that renders an partial element with a "wedge" shape, to be used
/// as part of the rim of the color wheel.
///
/// Note that this returns a `<g>` (group) element, not a full SVG, so it must be
/// embedded in an `<svg>` element to render properly.
pub fn Wedge(cx: Scope<WedgeProps>) -> Element {
  let props = cx.props;
  let color = props.color.to_hex_color();
  let text_color = props.text_color.to_hex_color();
  let end_angle = Angle::Degrees(props.arc_angle.as_degrees() / 2.0);
  let start_angle = Angle::Degrees(-(end_angle.as_degrees()));
  let p = polar_to_cartesian(props.center, props.radius, end_angle);
  let label_pt = polar_to_cartesian(props.center, props.radius * 0.9, 0.0.into());

  let wedge_path = vec![
    arc_svg_path(props.center, props.radius, start_angle, end_angle),
    line_to(props.center),
    line_to(p),
  ]
  .join(" ");

  let group_transform = format!(
    "rotate({}, {}, {})",
    props.rotation, props.center.x, props.center.y
  );
  cx.render(rsx! {
    g {
      transform: "{group_transform}",
      fill: "{color}",
      stroke: "{color}",
      key: "{props.label}",

      path {
        d: "{wedge_path}",
        stroke_width: "0",
        stroke: "none",
      }

      text {
        text_anchor: "middle",
        x: "{label_pt.x}",
        y: "{label_pt.y}",
        stroke: "{text_color}",
        fill: "{text_color}",

        "{props.label}"
      }
    }
  })
}
