use tune::note::NoteLetter;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteName {
  /// A named note in the traditional 12-TET musical notation system.
  /// Note that notes with "simple" names may represent non-traditional pitches
  /// in some tunings and temperments.
  /// 
  /// Equivalent to `UpDown(letter, 0)`
  Simple(NoteLetter),

  /// A note in [Ups and Downs notation](https://en.xen.wiki/w/Ups_and_downs_notation).
  /// 
  /// The integer component is the number of "up" or "down" EDO-steps to offset from the NoteLetter.
  /// e.g. `^C#` (up C sharp) would be `UpDown(NoteLetter::Csh, 1)`, 
  /// while `vvC#` (double down C sharp) would be `UpDown(NoteLetter::Csh, -2)`
  /// 
  /// See the convenience variants [Up], [Dup], [Down], and [Dud] for the common
  /// cases of single and double offsets.
  UpDown(NoteLetter, i8),

  /// An "up" note that sounds one EDO-step above the given NoteLetter.
  /// 
  /// Equivalent to `UpDown(letter, 1)`
  Up(NoteLetter),

  /// A "double up" note that sounds two EDO-steps above the given NoteLetter.
  /// 
  /// Equivalent to `UpDown(letter, 2)`
  Dup(NoteLetter),

  /// A "down" note that sounds one EDO-step below the given NoteLetter.
  /// 
  /// Equivalent to `UpDown(letter, -1)`
  Down(NoteLetter),

  /// A "double down" note that sounds two EDO-steps below the given NoteLetter.
  /// 
  /// Equivalent to `UpDown(letter, -2)`
  Dud(NoteLetter),
}

impl NoteName {
  /// Returns self as a [NoteName::UpDown] variant.
  /// 
  /// ```rust
  /// use tune::note::NoteLetter;
  /// use lumatone_keymap::harmony::NoteName;
  /// 
  /// let dup_c = NoteName::Dup(NoteLetter::C);
  /// assert_eq!(dup_c.as_up_down(), NoteName::UpDown(NoteLetter::C, 2));
  /// ```  
  pub fn as_up_down(&self) -> NoteName {
    use NoteName::*;
    match self {
        Simple(l) => UpDown(*l, 0),
        UpDown(l, o) => UpDown(*l, *o),
        Up(l) => UpDown(*l, 1),
        Dup(l) => UpDown(*l, 2),
        Down(l) => UpDown(*l, -1),
        Dud(l) => UpDown(*l, -2),
    }
  }

  /// Returns the "simplest" variant that can represent self.
  /// 
  /// ```rust
  /// use tune::note::NoteLetter;
  /// use lumatone_keymap::harmony::NoteName;
  /// 
  /// let up_down_dup_c = NoteName::UpDown(NoteLetter::C, 2);
  /// assert_eq!(up_down_dup_c.simplified(), NoteName::Dup(NoteLetter::C));
  /// 
  /// let up_down_c = NoteName::UpDown(NoteLetter::C, 0);
  /// assert_eq!(up_down_c.simplified(), NoteName::Simple(NoteLetter::C));
  /// ```
  pub fn simplified(&self) -> NoteName {
    use NoteName::*;
    let (letter, offset) = match self.as_up_down() {
      UpDown(letter, offset) => (letter, offset),
      _ => unreachable!(),
    };

    match offset {
      0 => Simple(letter),
      1 => Up(letter),
      2 => Dup(letter),
      -1 => Down(letter),
      -2 => Dud(letter),
      o => UpDown(letter, o)
    }
  }
}

