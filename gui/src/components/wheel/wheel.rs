
use dioxus::prelude::*;

use crate::{
  components::wheel::{wedge::Wedge, constellation::PitchConstellation},
  drawing::{Angle, Float, Point},
  harmony::view_model::{Tuning, Scale}, hooks::useuniqueid::use_unique_id,
};

#[derive(PartialEq, Props)]
pub struct WheelProps {
  pub radius: Float,
  pub tuning: Tuning,
  pub scale: Scale,
}

/// A component that renders a colored [chromatic circle](https://en.wikipedia.org/wiki/Chromatic_circle),
/// where each color is mapped to a pitch class in some musical tuning. 
/// In the center of the wheel, a pitch constellation shows which notes are included in the current scale.
pub fn ColorWheel(cx: Scope<WheelProps>) -> Element {
  let container_id = use_unique_id(cx, "color-wheel");

  let tuning = &cx.props.tuning;
  let scale = &cx.props.scale;
  let divisions = tuning.divisions();

  let r = cx.props.radius;
  let size = r * 2.0;
  let center = Point { x: r, y: r };
  let hole_radius = r * 0.8;

  let arc_angle = Angle::Degrees(360.0 / (divisions as f64));
  let ring_rotation = match tuning.pitch_class_index(scale.tonic()) {
    Some(i) => -(arc_angle.as_degrees() * (i as Float)),
    _ => 0.0,
  };

  // render all the wedges
  let wedges = (0..divisions).map(|i| {
    let rotation: Float = arc_angle.as_degrees() * (i as Float);
    let color = tuning.get_color(i);
    let text_color = tuning.get_text_color(i);
    let pc = tuning.get_pitch_class(i);
    let label = pc.name();

    rsx! {
      Wedge {
        key: "{label}",
        radius: r,
        center: center,
        rotation: rotation,
        arc_angle: arc_angle,
        color: color,
        text_color: text_color,
        label: String::from(label),
      }
    }
  });

  cx.render(rsx! {
    div {
      id: "{container_id}",
      width: "{size}px",
      height: "{size}px",
      display: "flex",
      align_items: "center",
      justify_content: "center",

      svg {
        width: "{size}px",
        height: "{size}px",

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
          transform: "rotate({ring_rotation}, {center.x}, {center.y})",
          g {
            mask: "url(#rim-clip)",
            wedges
          }

          PitchConstellation {
            radius: hole_radius,
            center: center,
            tuning: tuning,
            scale: scale,
          }
        }
      }
    }
  })
}