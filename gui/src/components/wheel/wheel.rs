
use dioxus::prelude::*;

use crate::{
  components::wheel::{wedge::Wedge, constellation::PitchConstellation},
  drawing::{Angle, Float, Point},
  harmony::view_model::{Tuning, Scale}, hooks::{useuniqueid::use_unique_id, usesizeobserver::use_size_observer},
};

#[derive(PartialEq, Props)]
pub struct WheelProps {
  pub tuning: Tuning,
  pub scale: Scale,
}

/// A component that renders a colored [chromatic circle](https://en.wikipedia.org/wiki/Chromatic_circle),
/// where each color is mapped to a pitch class in some musical tuning. 
/// In the center of the wheel, a pitch constellation shows which notes are included in the current scale.
pub fn ColorWheel(cx: Scope<WheelProps>) -> Element {
  let container_id_ref = use_unique_id(cx, "color-wheel");
  let container_id = container_id_ref.read().clone();
  let container_size = use_size_observer(cx, container_id.clone());
  println!("wheel container size: {:?}", container_size);

  let tuning = &cx.props.tuning;
  let scale = &cx.props.scale;
  let divisions = tuning.divisions();
  let default_radius = 300.0;

  // if the container div has an observed size (meaning it's actually been rendered),
  // constrain the radius to fit within it. Otherwise, use the radius from props.
  let r = match *container_size.current() {
    Some((w, h)) => {
      let min = f64::min(w, h);
      min / 2.0
    },
    None => default_radius 
  };
  let size = r * 2.0;

  let scale_factor = r/ default_radius;
  let font_size = format!("{}em", 1.0 * scale_factor);
  println!("wheel radius: {r} - scale: {scale_factor}");

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
      key: "{container_id}",
      id: "{container_id}",
      overflow: "hidden",
      width: "100%",
      height: "100%",
      font_size: "{font_size}",

      svg {
        width: "100%",
        height: "100%",
        view_box: "0 0 {size} {size}",

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
          
          PitchConstellation {
            radius: hole_radius,
            center: center,
            tuning: tuning,
            scale: scale,
          }
          
          g {
            mask: "url(#rim-clip)",
            g {
              wedges
            }
          }


        }
      }
  }
  })
}
