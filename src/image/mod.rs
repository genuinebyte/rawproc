mod cfa;

pub use cfa::CFA;

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

pub struct ComponentImage<T: Copy> {
	pub rgb: Vec<T>,
	pub meta: Metadata
}

use std::ops::Range;
use std::iter::StepBy;

impl<T: Copy> ComponentImage<T> {
	pub fn pixel_range(&self) -> StepBy<Range<usize>> {
		(0..(self.meta.width as usize * self.meta.height as usize * 3 as usize)).step_by(3)
	}

	pub fn component_range(&self) -> Range<usize> {
		0..(self.meta.width as usize * self.meta.height as usize * 3 as usize)
	}

	pub fn component(&self, x: u32, y: u32, color: Color) -> T {
		self.rgb[self.meta.xytoi(x, y) * 3 + color as usize]
	}

	pub fn set_component(&mut self, i: usize, color: Color, data: T) {
		self.rgb[i * 3 + color as usize] = data;
	}
}

impl ComponentImage<u16> {
	pub fn as_floats(self) -> ComponentImage<f32> {
		ComponentImage {
			rgb: self.rgb.into_iter().map(|x| -> f32 {
					x as f32/ 4096.0 //TODO: 4096 allow changing bitness
				}).collect(),
			meta: self.meta
		}
	}
}

impl ComponentImage<f32> {
	pub fn as_bytes(self) -> Vec<u8> {
		self.rgb.into_iter().map(|x| -> u8 {
			(x * 256.0) as u8 //TODO: 4096 allow changing bitness
		}).collect()
	}
}