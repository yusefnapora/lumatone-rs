
/// A coordinate on a hex grid, using the "axial" coordinate representation
/// described at https://www.redblobgames.com/grids/hexagons/#coordinates
/// The third `s` coordinate used by the "cube" representation is available
/// via the `s()` accessor method.
#[derive(Debug, PartialEq)]
pub struct Hex {
  q: i64,
  r: i64,
}

impl Hex {
  pub const fn new(q: i64, r: i64) -> Hex {
    Hex { q, r }
  }


  pub fn q(&self) -> i64 {
    self.q
  }

  pub fn r(&self) -> i64 {
    self.r
  }

  pub fn s(&self) -> i64 {
    -self.q - self.r
  }

	pub fn to_fractional(&self) -> FractionalHex {
		FractionalHex {
			q: self.q as f64,
			r: self.r as f64,
		}
	}

  pub fn add(&self, other: Hex) -> Hex {
    Hex {
      q: self.q + other.q,
      r: self.r + other.r,
    }
  }

  pub fn subtract(&self, other: Hex) -> Hex {
    Hex {
      q: self.q - other.q,
      r: self.r - other.r,
    }
  }

  pub fn multiply(&self, other: Hex) -> Hex {
    Hex {
      q: self.q * other.q,
      r: self.r * other.r,
    }
  }

  pub fn length(hex: Hex) -> i64 {
    hex.q.abs() + hex.r.abs() + hex.s().abs()
  }

  pub fn distance(&self, other: Hex) -> i64 {
    Hex::length(self.subtract(other))
  }

  pub fn neighbor(&self, dir: HexDirection) -> Hex {
    self.add(*dir.hex())
  }

	pub fn line_to(&self, other: Hex) -> Vec<Hex> {
		let n = self.distance(other);
		// nudge the coords a bit to consistently push points that are
		// directly in-between a hex to one side
		// see https://www.redblobgames.com/grids/hexagons/implementation.html#fractionalhex
		let epsilon = FractionalHex::new(1e6, 1e6);
		let a = self.to_fractional().add(epsilon);
		let b = other.to_fractional().add(epsilon);
		let step = 1.0 / (i64::max(n, 1) as f64);

		(0..n).map(|i| {
			a.lerp(b, step * (i as f64)).round()
		}).collect()
	}
}

pub const fn hex(q: i64, r: i64) -> Hex {
  Hex::new(q, r)
}

#[repr(usize)]
#[derive(Copy, Clone)]
pub enum HexDirection {
	East,
	NorthEast,
	NorthWest,
	West,
	SouthWest,
	SouthEast,
}

impl HexDirection {
	pub fn hex(&self) -> &'static Hex {
		let index = *self as usize;
		direction(index)
	}
}

static HEX_DIRECTIONS: [Hex; 6] = {
  [
    hex(1, 0),
    hex(1, -1),
    hex(0, -1),
    hex(-1, 0),
    hex(-1, 1),
    hex(0,1),
  ]
};

fn direction(dir_index: usize) -> &'static Hex {
  let d = (6 + (dir_index % 6)) % 6;
  &HEX_DIRECTIONS[d]
}


pub struct FractionalHex {
	q: f64,
	r: f64,
}

impl FractionalHex {
	pub const fn new(q: f64, r: f64) -> FractionalHex {
		FractionalHex { q, r }
	}

	pub fn q(&self) -> f64 {
		self.q
	}

	pub fn r(&self) -> f64 {
		self.r
	}

	pub fn s(&self) -> f64 {
		-self.q - self.r
	}

	pub fn round(&self) -> Hex {
		let mut q = self.q.round();
		let mut r = self.r.round();
		let s = self.s().round();
		let q_diff = f64::abs(q - self.q);
		let r_diff = f64::abs(r - self.r);
		let s_diff = f64::abs(s - self.s());
		if q_diff > r_diff && q_diff > s_diff {
			q = -r - s;
		} else if r_diff > s_diff {
			r = -q - s;
		}

		Hex { q: q as i64, r: r as i64 }
	}

	pub fn lerp(&self, other: FractionalHex, t: f64) -> FractionalHex {
		FractionalHex {
			q: lerp(self.q, other.q, t),
			r: lerp(self.r, other.r, t),
		}
	}

  pub fn add(&self, other: FractionalHex) -> FractionalHex {
    FractionalHex {
      q: self.q + other.q,
      r: self.r + other.r,
    }
  }

  pub fn subtract(&self, other: FractionalHex) -> FractionalHex {
    FractionalHex {
      q: self.q - other.q,
      r: self.r - other.r,
    }
  }

  pub fn multiply(&self, other: FractionalHex) -> FractionalHex {
    FractionalHex {
      q: self.q * other.q,
      r: self.r * other.r,
    }
  }

  pub fn length(hex: FractionalHex) -> f64 {
    hex.q.abs() + hex.r.abs() + hex.s().abs()
  }

  pub fn distance(&self, other: FractionalHex) -> f64 {
    FractionalHex::length(self.subtract(other))
  }

}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
	a * (1.0-t) + b * t
}
