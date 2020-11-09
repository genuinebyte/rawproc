pub enum CFA {
	/*
	R G R G
	G B G B
	R G R G
	G B G B
	*/
	RGGB
}

impl CFA {
	pub fn color_at(&self, x: u32, y: u32) -> Color {
		match self {
			CFA::RGGB => {
				if x%2 == 0 {
					if y%2 == 0 {
						return Color::Red;
					} else {
						return Color::Green;
					}
				} else {
					if y%2 == 0 {
						return Color::Green;
					} else {
						return Color::Blue;
					}
				}
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Color {
	Red,
	Green,
	Blue
}

pub struct Metadata {
	cfa: CFA,
	pub width: u32,
	pub height: u32
}

impl Metadata {
	pub fn new(cfa: CFA, width: u32, height: u32) -> Self {
		Self {
			cfa,
			width,
			height
		}
	}

	pub fn rgb_len(&self) -> usize {
		self.width as usize * self.height as usize * 3
	}

	pub fn are_coords_valid(&self, x: u32, y: u32) -> bool {
		if x < self.width && y < self.height {
			true
		} else {
			false
		}
	}

	pub fn color_at_xy(&self, x: u32, y: u32) -> Color {
		self.cfa.color_at(x, y)
	}

	pub fn color_at(&self, i: u32) -> Color {
		self.cfa.color_at(i % self.width, i / self.width)
	}
}

pub struct RawImage {
	pub raw: Vec<u16>,
	pub meta: Metadata
}

pub struct ComponentImage {
	pub rgb: Vec<u16>,
	pub meta: Metadata
}

impl ComponentImage {
	pub fn as_bytes(self) -> Vec<u8> {
		self.rgb.into_iter().map(|x| -> u8 {
			((x as f32/ 4096.0) * 256.0) as u8 //TODO: 4096 allow changing bitness
		}).collect()
	}
}

#[cfg(test)]
mod cfa_tets {
	use super::*;

	#[test]
	fn color_at_rggb() {
		// Testing initial pattern
		assert_eq!(CFA::RGGB.color_at(0, 0), Color::Red);
		assert_eq!(CFA::RGGB.color_at(1, 0), Color::Green);
		assert_eq!(CFA::RGGB.color_at(0, 1), Color::Green);
		assert_eq!(CFA::RGGB.color_at(1, 1), Color::Blue);

		// Testing expanded pattern
		assert_eq!(CFA::RGGB.color_at(2, 2), Color::Red);
		assert_eq!(CFA::RGGB.color_at(3, 2), Color::Green);
		assert_eq!(CFA::RGGB.color_at(2, 3), Color::Green);
		assert_eq!(CFA::RGGB.color_at(3, 3), Color::Blue);
	}
}