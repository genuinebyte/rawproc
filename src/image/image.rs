use crate::Color;
use std::ops::Range;
use std::iter::StepBy;
use crate::CFA;
use libraw::ColorData;

pub struct Metadata {
	pub width: u32,
	pub height: u32,
	pub cfa: CFA,
	pub colordata: ColorData
}

impl Metadata {
	pub fn new(width: u32, height: u32, cfa: CFA, colordata: ColorData) -> Self {
		Self {
			width,
			height,
			cfa,
			colordata
		}
	}

	pub fn xytoi(&self, x: u32, y: u32) -> usize {
		(y * self.width + x) as usize
	}

	pub fn itoxy(&self, i: usize) -> (u32, u32) {
		let y = i / self.width as usize;
		let x = i % self.width as usize;

		(x as u32, y as u32)
	}

	pub fn pixels(&self) -> usize {
		self.width as usize * self.height as usize
	}

	pub fn color_at_index(&self, index: usize) -> Color {
		let (x, y) = self.itoxy(index);
		self.cfa.color_at(x, y)
	}

	pub fn color_at_xy(&self,x: u32, y: u32) -> Color {
		self.cfa.color_at(x, y)
	}
}

pub struct RawImage {
	pub(crate) raw: Vec<u16>,
	pub meta: Metadata
}

impl RawImage {
}

pub struct RgbImage<T: Copy> {
	pub(crate) rgb: Vec<T>,
	pub meta: Metadata
}

impl<T: Copy> RgbImage<T> {
	pub fn pixel_range(&self) -> StepBy<Range<usize>> {
		self.component_range().step_by(3)
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

impl RgbImage<u16> {
	pub fn as_floats(self) -> RgbImage<f32> {
		RgbImage {
			rgb: self.rgb.into_iter().map(|x| -> f32 {
					x as f32/ 4096.0 //TODO: 4096 allow changing bitness
				}).collect(),
			meta: self.meta
		}
	}
}

impl RgbImage<f32> {
	pub fn as_bytes(self) -> Vec<u8> {
		self.rgb.into_iter().map(|x| -> u8 {
			(x * 256.0) as u8
		}).collect()
	}
}