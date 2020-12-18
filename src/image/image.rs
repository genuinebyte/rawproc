use crate::Color;
use crate::Metadata;
use std::ops::Range;
use std::iter::StepBy;

pub struct RgbImage<T: Copy> {
	pub rgb: Vec<T>,
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