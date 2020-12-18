mod cfa;
mod image;

pub use cfa::CFA;
pub use image::RgbImage;

use crate::Color;

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

	pub fn xytoi(&self, x: u32, y: u32) -> usize {
		(y * self.width + x) as usize
	}

	pub fn itoxy(&self, i: usize) -> (u32, u32) {
		let y = i / self.width as usize;
		let x = i % self.width as usize;

		(x as u32, y as u32)
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

	pub fn color_at(&self, i: usize) -> Color {
		self.cfa.color_at((i % self.width as usize) as u32, (i / self.width as usize) as u32)
	}
}

pub struct RawImage {
	pub raw: Vec<u16>,
	pub meta: Metadata
}