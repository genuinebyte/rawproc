mod cfa;

pub use cfa::CFA;

use num_traits::Num;

#[derive(Debug, PartialEq)]
pub enum Color {
	Red,
	Green,
	Blue
}

pub struct Pattern {
	cfa: CFA,
	width: u32,
	height: u32
}

impl Pattern {
	pub fn new(cfa: CFA, width: u32, height: u32) -> Self {
		Pattern {
			cfa,
			width,
			height
		}
	}

	pub fn color_at_xy(&self, x: u32, y: u32) -> Color {
		self.cfa.color_at(x, y)
	}

	pub fn color_at(&self, i: u32) -> Color {
		self.cfa.color_at(i % self.width, i / self.width)
	}
}

pub struct Debayer<'a, T: Num + Copy> {
	pattern: Pattern,
	width: u32,
	height: u32,
	rgb: &'a mut [T]
}

impl<'a, T: Num + Copy> Debayer<'a, T> {
	pub fn new(raw: &[T], rgb: &'a mut [T], cfa: CFA, width: u32, height: u32) -> Self {
		let pattern = Pattern::new(cfa, width, height);

		let capacity = (width * height * 3) as usize;
		
		if rgb.len() != capacity {
			panic!("Expected buffer of size {} but got {}", capacity, rgb.len());
		}

		let mut i: usize = 0;
		for light in raw {
			let light = *light;
			match pattern.color_at(i as u32) {
				Color::Red => {
					rgb[i*3] = light;
					rgb[i*3 + 1] = light - light;
					rgb[i*3 + 2] = light - light;
				},
				Color::Green => {
					rgb[i*3] = light - light;
					rgb[i*3 + 1] = light;
					rgb[i*3 + 2] = light - light;
				},
				Color::Blue => {
					rgb[i*3] = light - light;
					rgb[i*3 + 1] = light - light;
					rgb[i*3 + 2] = light;
				}
			}

			i += 1;
		}

		Self {
			pattern,
			width,
			height,
			rgb
		}
	}

	fn get_pixel(&self, color: Color, x: u32, y: u32) -> &T {
		if x > self.width || y > self.height {
			panic!(format!("Tried to acces a pixel out of the images bounds: {}, {}", x, y));
		}

		let index = ((y * self.width + x) * 3) as usize;
		unsafe {
			match color {
				Color::Red => {
					self.rgb.get_unchecked(index)
				},
				Color::Green => {
					self.rgb.get_unchecked(index+1)
				},
				Color::Blue => {
					self.rgb.get_unchecked(index+2)
				}
			}
		}
	}

	fn set_pixel(&mut self, color: Color, value: T, x: u32, y: u32) {
		if x > self.width || y > self.height {
			panic!("Tried to set a pixel out of the images bounds!");
		}

		let index = ((y * self.width + x) * 3) as usize;
		unsafe {
			match color {
				Color::Red => {
					*self.rgb.get_unchecked_mut(index) = value;
				},
				Color::Green => {
					*self.rgb.get_unchecked_mut(index+1) = value;
				},
				Color::Blue => {
					*self.rgb.get_unchecked_mut(index+2) = value;
				}
			}
		}
	}

	//FIXME: This isn't proper nearest neighboor at all, but uh, it's good enough for now?
	pub unsafe fn nearest_neighboor(&mut self) {
		let mut red = *self.rgb.get_unchecked(0);
		let mut green = *self.rgb.get_unchecked(1);
		let mut blue = *self.rgb.get_unchecked(2);

		for x in 0..self.width {
			for y in 0..self.height {
				unsafe {
					match self.pattern.color_at_xy(x, y) {
						Color::Red => {
							red = *self.get_pixel(Color::Red, x, y);
							self.set_pixel(Color::Green, green, x, y);
							self.set_pixel(Color::Blue, blue, x, y);
						},
						Color::Green => {
							self.set_pixel(Color::Red, red, x, y);
							green = *self.get_pixel(Color::Green, x, y);
							self.set_pixel(Color::Blue, blue, x, y);
						},
						Color::Blue => {
							self.set_pixel(Color::Red, red, x, y);
							self.set_pixel(Color::Green, green, x, y);
							blue = *self.get_pixel(Color::Blue, x, y);
						},
					}
				}
			}
		}
	}
}