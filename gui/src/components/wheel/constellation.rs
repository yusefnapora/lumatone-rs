//! A [pitch constellation](https://en.wikipedia.org/wiki/Chromatic_circle#Pitch_constellation)
//! that's rendered in the center of the color wheel / chromatic circle component.

use dioxus::prelude::*;
use palette::LinSrgb;

use lumatone_core::color::utils::ToHexColorStr;
use lumatone_core::geometry::{polar_to_cartesian, Angle, Float, Point};
use crate::{
  harmony::view_model::{Scale, Tuning},
};

#[derive(PartialEq, Props)]
pub struct ConstellationProps<'a> {
  radius: Float,
  center: Point,
  opacity: Option<Float>,
  tuning: &'a Tuning,
  scale: &'a Scale,
}

pub fn PitchConstellation<'a>(cx: Scope<'a, ConstellationProps<'a>>) -> Element {
  let radius = cx.props.radius;
  let center = cx.props.center;
  let opacity = cx.props.opacity.unwrap_or(0.6);
  let tuning = cx.props.tuning;
  let scale = cx.props.scale;

  let degrees_per_division = 360.0 / tuning.divisions() as f64;
  let stroke_width = radius * 0.25;

  // loop over all pitch classes in the tuning and render `<line>` elements
  // for each scale tone
  let lines = (0..tuning.divisions()).map(|i| {
    let pc = tuning.get_pitch_class(i);
    let key = pc.name();
    // skip non scale tones
    if !scale.contains(pc) {
      return rsx! { g { key: "{key}" } };
    }

    let angle = degrees_per_division * (i as f64);
    let color = tuning.get_color(i);

    rsx! {
      PitchLine {
        key: "{key}",
        center: center,
        angle: angle,
        radius: radius,
        stroke_width: stroke_width,
        opacity: opacity,
        color: color,
      }
    }
  });

  // wrap all the lines in a <g> group element & return
  cx.render(rsx! {
    g { lines }
  })
}

#[derive(PartialEq, Props)]
struct PitchLineProps {
  #[props(into)]
  center: Point,
  radius: Float,
  angle: Float,
  stroke_width: Float,
  opacity: Float,
  color: LinSrgb,
}

fn PitchLine(cx: Scope<PitchLineProps>) -> Element {
  let p = cx.props;
  let end_point = polar_to_cartesian(p.center, p.radius, Angle::Degrees(p.angle));
  let color = p.color.to_hex_color();

  cx.render(rsx! {
    line {
      x1: p.center.x,
      y1: p.center.y,
      x2: end_point.x,
      y2: end_point.y,
      stroke: "{color}",
      fill: "{color}",
      stroke_width: p.stroke_width,
      stroke_linecap: "round",
      opacity: p.opacity,
    }
  })
}
