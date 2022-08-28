/// A Dioxus component that renders a color wheel, where each color is mapped to a
/// pitch class in some musical scale. In the center of the wheel, a pitch constellation
/// shows which notes are included in the scale.

use dioxus::prelude::*;

use crate::drawing::{Angle, Float, Point, polar_to_cartesian, arc_svg_path, line_to};

pub fn ColorWheel(cx: Scope) -> Element {
  // TODO: everything
  cx.render(rsx! {
    svg {
      circle {
        cx: "100",
        cy: "100",
        r: "50",
        stroke: "#000",
      }
    }
  })
}

// TODO: use color type from palette crate
type Color = String;

struct WedgeProps {
  radius: Float,
  center: Point,
  color: Color, 
  text_color: Color,
  label: String,
  rotation: Float,
  arc_angle: Angle,
}

/// A component that renders an partial element with a "wedge" shape, to be used
/// as part of the rim of the color wheel.
/// Note that this returns a <G> (group) element, not a full SVG, so it must be
/// embedded in an <SVG> element to render properly.
fn Wedge(cx: Scope<WedgeProps>) -> Element {
  let props = cx.props;
  let end_angle = Angle::Degrees(props.arc_angle.as_degrees() / 2.0);
  let start_angle = Angle::Degrees(-end_angle.as_degrees());
  let p = polar_to_cartesian(props.center, props.radius, end_angle);
  let label_pt = polar_to_cartesian(props.center, props.radius, 0.0.into());
  
  let wedge_path = vec![
    arc_svg_path(props.center, props.radius, start_angle, end_angle),
    line_to(props.center),
    line_to(p),
  ].join(" ");

  let group_transform = format!("rotate({}, {}, {})", props.rotation, props.center.x, props.center.y);
  cx.render(rsx! {
    g {
      transform: "{group_transform}",
      fill: "{props.color}",
      stroke: "{props.color}",
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
        stroke: "{props.text_color}",
        fill: "{props.text_color}",

        "{props.label}"
      }
    }
  })
}