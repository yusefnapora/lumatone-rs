/// A Dioxus component that renders a color wheel, where each color is mapped to a
/// pitch class in some musical scale. In the center of the wheel, a pitch constellation
/// shows which notes are included in the scale.
use dioxus::prelude::*;

use crate::drawing::{arc_svg_path, line_to, polar_to_cartesian, Angle, Float, Point};

#[derive(PartialEq, Props)]
pub struct Props {
  pub radius: Float,
  // TODO: add props for color palette, scale / harmonic structure, etc
}

pub fn ColorWheel(cx: Scope<Props>) -> Element {
  // TODO: convert divisions, scale, etc to props
  let divisions = 12;
  let colors = vec![
    "#ff0000", "#bf0041", "#800080", "#55308d", "#2a6099", "#158466", "#00a933", "#81d41a",
    "#ffff00", "#ffbf00", "#ff8000", "#ff4000",
  ];
  let labels = vec![
    "C", "C# / Db", "D", "D# / Eb", "E", "F", "F# / Gb", "G", "G# / Ab", "A", "A# / Bb", "B",
  ];

  let r = cx.props.radius;
  let center = Point { x: r, y: r };
  let hole_radius = r * 0.8;
  let mut wedges = vec![];

  let arc_angle = Angle::Degrees(30.0); // TODO: 360.0 / divisions
  let ring_rotation = 0.0; // TODO: rotate so tonic of current scale is north

  for i in 0..divisions as usize {
    let rotation: Float = arc_angle.as_degrees() * (i as Float);
    let color = colors[i].to_string();
    let text_color = "#000000".to_string(); // TODO: use complement of main color
    let label = labels[i].to_string();
    let wedge = rsx!(Wedge {
      radius: r,
      center: center,
      rotation: rotation,
      arc_angle: arc_angle,
      color: color,
      text_color: text_color,
      label: label
    });
    wedges.push(wedge);
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
          wedges
        }
      }
    }
  })
}

// TODO: use color type from palette crate
type Color = String;

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
