// WIP view models for tuning & scales. needs a lot of revision to fully cover the domain

pub mod view_model {

  use crate::drawing::color::ColorPalette;
  use palette::LinSrgb;

  #[derive(PartialEq)]
  pub struct PitchClass {
    pub name: String,
    // TODO: add optional enharmonic name(s)
  }

  #[derive(PartialEq)]
  pub struct Tuning {
    pub name: String,
    pitch_classes: Vec<PitchClass>,
    palette: ColorPalette,
  }

  impl Tuning {
    pub fn new(name: String, pitch_classes: Vec<PitchClass>) -> Tuning {
      let palette = ColorPalette::default_gradient(pitch_classes.len());
      Tuning {
        name,
        pitch_classes,
        palette,
      }
    }

    pub fn edo_12() -> Tuning {
      let name = "12 EDO";
      let pitch_classes = vec![
        PitchClass {
          name: String::from("C"),
        },
        PitchClass {
          name: String::from("C#"),
        },
        PitchClass {
          name: String::from("D"),
        },
        PitchClass {
          name: String::from("D#"),
        },
        PitchClass {
          name: String::from("E"),
        },
        PitchClass {
          name: String::from("F"),
        },
        PitchClass {
          name: String::from("F#"),
        },
        PitchClass {
          name: String::from("G"),
        },
        PitchClass {
          name: String::from("G#"),
        },
        PitchClass {
          name: String::from("A"),
        },
        PitchClass {
          name: String::from("A#"),
        },
        PitchClass {
          name: String::from("B"),
        },
      ];
      Tuning::new(String::from(name), pitch_classes)
    }

    pub fn divisions(&self) -> usize {
      self.pitch_classes.len()
    }

    pub fn get_pitch_class(&self, index: usize) -> &PitchClass {
      &self.pitch_classes[index]
    }

    pub fn get_color(&self, index: usize) -> LinSrgb {
      self.palette.get(index)
    }

    pub fn get_text_color(&self, index: usize) -> LinSrgb {
      self.palette.get_text_color(index)
    }
  }
}
