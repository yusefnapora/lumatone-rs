use core::hash::Hasher;
use std::{collections::{HashSet, HashMap}, hash::Hash, ops::Deref, fmt::Debug};
use hexagon_tiles::hexagon::{Hex as _Hex, HexMath};
pub use hexagon_tiles::hexagon::FractionalHex;
use lumatone_midi::constants::{LumatoneKeyIndex, BoardIndex, LumatoneKeyLocation};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Hex(_Hex);

impl Hex {
	pub fn new(q: i32, r: i32) -> Hex {
		Hex(_Hex::new(q, r))
	}

	fn from_hextile_hex(h: _Hex) -> Hex {
		Hex::new(h.q(), h.r())
	}

	pub fn to_string(&self) -> String {
		format!("{}, {}, {}", self.q(), self.r(), self.s())
	}

	fn add(&self, other: Hex) -> Hex {
		Hex::from_hextile_hex(self.0.add(other.0))
	}

	fn sub(&self, other: Hex) -> Hex {
		Hex::from_hextile_hex(self.0.sub(other.0))
	}

	fn scale(&self, k: i32) -> Hex {
		Hex::from_hextile_hex(self.0.scale(k))
	}
}

impl Deref for Hex {
	type Target = _Hex;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}



impl Debug for Hex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hex")
					.field("q", &self.0.q())
					.field("r", &self.0.r())
					.field("s", &self.0.s())
					.finish()
    }
}

impl Hash for Hex {
	fn hash<H: Hasher>(&self, h: &mut H) {
		h.write_i32(self.q());
		h.write_i32(self.r());
		h.write_i32(self.s());
		h.finish();
	}
}

/// Generates Hex coordinates that cover a 56-key "octave" section of the board.
pub fn gen_octave_coords(octave_num: u8) -> Vec<Hex> {
	const BOARD_OFFSET_COL: u8 = 5;
	const BOARD_OFFSET_ROW: u8 = 2;

	let mut coords = Vec::with_capacity(56);
	let start_col = 0; // + (BOARD_OFFSET_COL * octave_num) as i32;
	let start_row = 0; // + (BOARD_OFFSET_ROW * octave_num) as i32;
	let end_col = start_col + 6;
	let end_row = start_row + 11;

	for r in start_row..end_row {
		// special case the first and last two rows to account for missing keys
		let (start_col, end_col) = match r {
			0 => (0, 2),
			1 => (0, 5),
			9 => (1, 6),
		  10 => (4, 6),
			_ => (start_col, end_col)
		};
		let r_offset = (r as f64 / 2.0).floor() as i32;
		
		let r = r + (BOARD_OFFSET_ROW * octave_num) as i32;
		let start_col = start_col + (BOARD_OFFSET_COL * octave_num) as i32;
		let end_col = end_col + (BOARD_OFFSET_COL * octave_num) as i32;
				
		let start_col = start_col - r_offset;
		let end_col = end_col - r_offset;
		for q in start_col..end_col {
			coords.push(Hex::new(q, r));
		}
	}

	coords
}


/// Generates Hex coordinates the cover the full range of a Lumatone.
pub fn gen_full_board_coords() -> HashSet<Hex> {
	let mut s = HashSet::with_capacity(280);
	for i in 0..5 {
		s.extend(gen_octave_coords(i));
	}
	s
}

// TOOD: use lazy_static to gerate this mapping once
pub struct LumatoneCoordinateMapping {
	from_lumatone: HashMap<LumatoneKeyLocation, Hex>,
	from_hex: HashMap<Hex, LumatoneKeyLocation>,
}

impl LumatoneCoordinateMapping {
	pub fn new() -> LumatoneCoordinateMapping {
		let mut from_lumatone= HashMap::with_capacity(280);
		let mut from_hex = HashMap::with_capacity(280);
		for i in 0..5 {
			let board_index = BoardIndex::try_from(i+1).expect("invalid board index");
			let coords = gen_octave_coords(i);
			for (k, hex) in coords.iter().enumerate() {
				let key_index = LumatoneKeyIndex::try_from(k as u8).expect("invalid key index");
				let location = LumatoneKeyLocation(board_index, key_index);
				from_lumatone.insert(location, *hex);
				from_hex.insert(*hex, location);
			}
		}
		LumatoneCoordinateMapping { from_lumatone, from_hex }
	}

	pub fn get_hex(&self, location: &LumatoneKeyLocation) -> &Hex {
		self.from_lumatone.get(location).unwrap()
	}

	pub fn get_lumatone_key(&self, hex: &Hex) -> Option<&LumatoneKeyLocation> {
		self.from_hex.get(hex)
	}
}

