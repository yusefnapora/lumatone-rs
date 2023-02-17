//! WIP view models for tuning & scales. needs a lot of revision to fully cover the domain
use std::collections::HashSet;

use crate::drawing::color::ColorPalette;
use palette::LinSrgb;

#[derive(Hash, Eq, PartialEq)]
pub struct PitchClass {
  name: String,
  // TODO: add optional enharmonic name(s)
}

impl PitchClass {
  pub fn name(&self) -> &str {
    &self.name
  }
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

  pub fn pitch_class_index(&self, pc: &PitchClass) -> Option<usize> {
    for (i, p) in self.pitch_classes.iter().enumerate() {
      if pc == p {
        return Some(i)
      }
    }
    None
  }
}

#[derive(PartialEq)]
pub struct Scale {
  name: String,
  // TODO: optional vec of alternate names

  tonic: PitchClass,
  scale_tones: HashSet<PitchClass>
}

impl Scale {
  pub fn new(name: String, tonic: PitchClass, scale_tones: HashSet<PitchClass>) -> Scale {
    Scale { name, tonic, scale_tones }
  }

  pub fn contains(&self, pc: &PitchClass) -> bool {
    self.scale_tones.contains(pc)
  }

  pub fn tonic(&self) -> &PitchClass {
    &self.tonic
  }

  // TODO: generate scales instead of hard-coding :)
  pub fn c_major() -> Scale {
    Scale {
      name: String::from("C major"),
      tonic: PitchClass { name: String::from("C") },
      scale_tones: HashSet::from([
        PitchClass { name: String::from("C") },
        PitchClass { name: String::from("D") },
        PitchClass { name: String::from("E") },
        PitchClass { name: String::from("F") },
        PitchClass { name: String::from("G") },
        PitchClass { name: String::from("A") },
        PitchClass { name: String::from("B") },
      ])
    }
  }
  
  pub fn d_major() -> Scale {
    Scale {
      name: String::from("C major"),
      tonic: PitchClass { name: String::from("D") },
      scale_tones: HashSet::from([
        PitchClass { name: String::from("D") },
        PitchClass { name: String::from("E") },
        PitchClass { name: String::from("F#") },
        PitchClass { name: String::from("G") },
        PitchClass { name: String::from("A") },
        PitchClass { name: String::from("B") },
        PitchClass { name: String::from("C#") },
      ])
    }
  }}