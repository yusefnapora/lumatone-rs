/// A Dioxus component that renders a color wheel, where each color is mapped to a
/// pitch class in some musical scale. In the center of the wheel, a pitch constellation
/// shows which notes are included in the scale.
use dioxus::prelude::*;
use palette::LinSrgb;

use crate::{
  drawing::{arc_svg_path, color::ToHexColorStr, line_to, polar_to_cartesian, Angle, Float, Point},
  harmony::view_model::Tuning,
};

#[derive(PartialEq, Props)]
pub struct Props {
  pub radius: Float,
  pub tuning: Tuning,
}

pub fn ColorWheel(cx: Scope<Props>) -> Element {
  let tuning = &cx.props.tuning;
  let divisions = tuning.divisions();

  let r = cx.props.radius;
  let center = Point { x: r, y: r };
  let hole_radius = r * 0.8;
  let mut wedges = vec![];

  let arc_angle = Angle::Degrees(360.0 / (divisions as f64));
  let ring_rotation = 0.0; // TODO: rotate so tonic of current scale is north

  for i in 0..divisions {
    let rotation: Float = arc_angle.as_degrees() * (i as Float);
    let color = tuning.get_color(i);
    let text_color = tuning.get_text_color(i);
    let label = tuning.get_pitch_class(i).name.clone(); // TODO: use references instead of cloning
    let wedge_props = WedgeProps {
      radius: r,
      center: center,
      rotation: rotation,
      arc_angle: arc_angle,
      color: color,
      text_color: text_color,
      label: label.clone(),
    };
    wedges.push((wedge_props, label));
  }

  cx.render(rsx! {
    div {
      width: "100%",
      height: "100%",
      display: "flex",
      align_items: "center",
      justify_content: "center",

      svg {
        width: "inherit",
        height: "inherit",

        defs {
          // clipping mask to cut out the center of the wheel.
          // The white portion is rendered, black is removed.
          mask {
            id: "rim-clip",
            // cover the whole area in white
            circle {
              cx: "{center.x}",
              cy: "{center.y}",
              r: "{r}",
              fill: "white"
            }
            // overlap black in the center to mask out hole
            circle {
              cx: "{center.x}",
              cy: "{center.y}",
              r: "{hole_radius}",
              fill: "black"
            }
          }
        }

        g {
          mask: "url(#rim-clip)",
          transform: "rotate({ring_rotation}, {center.x}, {center.y})",
          for (w, key) in wedges.into_iter() {
            // TODO: figure out if it's possible to just pass in the props struct
            Wedge {
              key: "{key}",
              radius: w.radius,
              center: w.center,
              rotation: w.rotation,
              arc_angle: w.arc_angle,
              color: w.color,
              text_color: w.text_color,
              label: w.label,
            }
          }
        }
      }
    }
  })
}

type Color = LinSrgb;

#[derive(PartialEq, Props)]
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
///
/// Note that this returns a `<g>` (group) element, not a full SVG, so it must be
/// embedded in an `<svg>` element to render properly.
fn Wedge(cx: Scope<WedgeProps>) -> Element {
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
