use crate::Color;
use std::ops::Range;
use std::iter::StepBy;
use crate::CFA;
use libraw::Colordata;
use num_traits::{Num, PrimInt, AsPrimitive};
use crate::Processor;

pub struct Metadata {
	pub width: u32,
	pub height: u32,
	pub cfa: CFA,
	pub bit_depth: u8,
	pub colordata: Colordata
}

impl Metadata {
	pub fn new(width: u32, height: u32, cfa: CFA, colordata: Colordata) -> Self {
		Self {
			width,
			height,
			cfa,
			bit_depth: 12, //TODO: Allow changing bit depth
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

	pub fn depth_max(&self) -> usize {
		2.pow(self.bit_depth as u32) - 1
	}
}

pub trait Kind{
	fn per_pixel() -> usize;
}

pub struct Sensor{}
impl Kind for Sensor {
	fn per_pixel() -> usize {
		1
	}
}

pub struct Rgb{}
impl Kind for Rgb {
	fn per_pixel() -> usize {
		3
	}
}

pub struct Hsv{}
impl Kind for Hsv {
	fn per_pixel() -> usize {
		3
	}
}

pub trait Component: Num + Copy {}
impl<T: Num + Copy> Component for T {}

pub struct Image<K: Kind, T: Component> {
	pub kind: K,
	pub data: Vec<T>,
	pub meta: Metadata,
}

impl<K: Kind, T: Component> Image<K, T> {
	pub fn pixel_range(&self) -> Range<usize> {
		0..(self.meta.width as usize * self.meta.height as usize)
	}

	pub fn pixel_index_range(&self) -> StepBy<Range<usize>> {
		self.component_range().step_by(K::per_pixel())
	}

	pub fn component_range(&self) -> Range<usize> {
		0..self.data.len()
	}

	pub fn component<C: Into<usize>>(&self, x: u32, y: u32, component: C) -> T {
		self.data[self.meta.xytoi(x, y) * K::per_pixel() + component.into()]
	}

	pub fn set_component<C: Into<usize>>(&mut self, i: usize, component: C, value: T) {
		self.data[i * K::per_pixel() + component.into()] = value;
	}
}

impl<K: Kind, I: Component + PrimInt + AsPrimitive<f32>> Image<K, I> {
	pub fn to_floats(self) -> Image<K, f32> {
		let max = self.meta.depth_max() as f32;

		Image {
			kind: self.kind,
			data: self.data.into_iter().map(|x| -> f32 {
					x.as_() / max
				}).collect(),
			meta: self.meta
		}
	}
}

impl<K: Kind> Image<K, f32> {
	pub fn to_bytes(self) -> Image<K, u8> {
		Image {
			kind: self.kind,
			data: self.data.into_iter().map(|x| -> u8 {
					(x * 255.0) as u8
				}).collect(),
			meta: self.meta
		}
	}
}

impl From<Image<Rgb, f32>> for Image<Hsv, f32> {
	fn from(rgb: Image<Rgb, f32>) -> Image<Hsv, f32> {
		Processor::rgb_to_hsv(rgb)
	}
}

impl From<Image<Hsv, f32>> for Image<Rgb, f32> {
	fn from(hsv: Image<Hsv, f32>) -> Image<Rgb, f32> {
		Processor::hsv_to_rgb(hsv)
	}
}