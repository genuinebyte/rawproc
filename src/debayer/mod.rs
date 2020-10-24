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
	width: usize,
	height: usize
}

impl Pattern {
	pub fn new(cfa: CFA, width: usize, height: usize) -> Self {
		Pattern {
			cfa,
			width,
			height
		}
	}

	pub fn color_at_xy(&self, x: usize, y: usize) -> Color {
		self.cfa.color_at(x, y)
	}

	pub fn color_at(&self, i: usize) -> Color {
		self.cfa.color_at(i % self.width, i / self.width)
	}
}

pub struct Debayer<T: Num + Copy> {
	pattern: Pattern,
	width: usize,
	height: usize,
	red: Vec<T>,
	green: Vec<T>,
	blue: Vec<T>
}

impl<T: Num + Copy> Debayer<T> {
	pub fn new(raw: Vec<T>, cfa: CFA, width: usize, height: usize) -> Self {
		let pattern = Pattern::new(cfa, width, height);

		let mut red = Vec::with_capacity(width * height);
		let mut green = Vec::with_capacity(width * height);
		let mut blue = Vec::with_capacity(width * height);
		
		let mut i: usize = 0;
		for light in raw {
			match pattern.color_at(i) {
				Color::Red => {
					red.push(light);
					green.push(light - light);
					blue.push(light - light);
				},
				Color::Green => {
					red.push(light - light);
					green.push(light);
					blue.push(light - light);
				},
				Color::Blue => {
					red.push(light - light);
					green.push(light - light);
					blue.push(light);
				}
			}

			i += 1;
		}

		Self {
			pattern,
			width,
			height,
			red,
			green,
			blue
		}
	}

	pub fn get_red(&self) -> &[T] {
		self.red.as_slice()
	}

	pub fn get_green(&self) -> &[T] {
		self.green.as_slice()
	}

	pub fn get_blue(&self) -> &[T] {
		self.blue.as_slice()
	}

	fn get_pixel(&self, color: Color, x: usize, y: usize) -> &T {
		if x > self.width || y > self.height {
			panic!(format!("Tried to acces a pixel out of the images bounds: {}, {}", x, y));
		}

		unsafe {
			match color {
				Color::Red => {
					self.red.get_unchecked(y * self.width + x)
				},
				Color::Green => {
					self.green.get_unchecked(y * self.width + x)
				},
				Color::Blue => {
					self.blue.get_unchecked(y * self.width + x)
				}
			}
		}
	}

	fn set_pixel(&mut self, color: Color, value: T, x: usize, y: usize) {
		if x > self.width || y > self.height {
			panic!("Tried to set a pixel out of the images bounds!");
		}

		unsafe {
			match color {
				Color::Red => {
					*self.red.get_unchecked_mut(y * self.width + x) = value;
				},
				Color::Green => {
					*self.green.get_unchecked_mut(y * self.width + x) = value;
				},
				Color::Blue => {
					*self.blue.get_unchecked_mut(y * self.width + x) = value;
				}
			}
		}
	}

	//FIXME: This only works for RGGB
	fn closest_of_color(&self, color: Color, x: usize, y: usize) -> usize {
		let current = self.pattern.color_at_xy(x, y);
		let current_index = y * self.width + x;

		let is_left_edge = x < 1;
		let is_top_edge = y < 1;
		let is_odd_row = y % 2 == 1;
		match current {
			Color::Red => match color {
				Color::Red => current_index,
				Color::Green => if is_left_edge {
					// There are no greens to the left, get one from the left
					current_index+1
				} else {
					current_index-1
				},
				Color::Blue => if is_left_edge {
					// There are no blues to the left, can we get one from above?
					if is_top_edge {
						// No, we can't! Get one from below
						current_index + self.width + 1
					} else {
						current_index - self.width + 1
					}
				} else {
					// There are blues on the left, can we go above?
					if is_top_edge{
						// No! Get one from below
						current_index + self.width - 1
					} else {
						current_index - self.width - 1
					}
				}
			},
			Color::Green => match color {
				Color::Red => if is_odd_row {
					// There should always be a red above because the row is odd
					current_index - self.width
				} else {
					// The row is even, so there'll always be one to the left
					current_index - 1
				},
				Color::Green => current_index,
				Color::Blue => if is_odd_row {
					// The row is odd, so there are blues on the left or the right...
					if is_left_edge {
						// Unless this is the left edge, so go right
						current_index + 1
					} else {
						current_index - 1
					}
				} else {
					// The row isn't odd, so there are blues above and below..
					if is_top_edge {
						// Unless we're on top, so go below
						current_index + self.width
					} else {
						current_index - self.width
					}
				}
			},
			Color::Blue => match color {
				// There will ALWAYS be a red to the upper-left
				Color::Red => current_index - self.width - 1,
				// There will ALWAYS be a green above
				Color::Green => current_index - self.width,
				Color::Blue => current_index
			}
		}
	}

	pub fn nearest_neighboor(&mut self) {
		for x in 0..self.width {
			for y in 0..self.height {
				unsafe {
					match self.pattern.color_at_xy(x, y) {
						Color::Red => {
							// Set green and blue
							self.set_pixel(Color::Green, *self.green.get_unchecked(self.closest_of_color(Color::Green, x, y)), x, y);
							self.set_pixel(Color::Blue, *self.blue.get_unchecked(self.closest_of_color(Color::Blue, x, y)), x, y);
						},
						Color::Green => {
							// Set red and blue
							self.set_pixel(Color::Red, *self.red.get_unchecked(self.closest_of_color(Color::Red, x, y)), x, y);
							self.set_pixel(Color::Blue, *self.blue.get_unchecked(self.closest_of_color(Color::Blue, x, y)), x, y);
						},
						Color::Blue => {
							// Set red and green
							self.set_pixel(Color::Red, *self.red.get_unchecked(self.closest_of_color(Color::Red, x, y)), x, y);
							self.set_pixel(Color::Green, *self.green.get_unchecked(self.closest_of_color(Color::Green, x, y)), x, y);
						},
					}
				}
			}
		}
	}

	pub fn get_fullcolor(self) -> Vec<T> {
		let mut full = Vec::with_capacity(self.width * self.height * 3);

		for i in 0..full.capacity() {
			unsafe {
				match i % 3 {
					0 => full.push(*self.red.get(i/3).unwrap()),
					1 => full.push(*self.green.get(i/3).unwrap()),
					2 => full.push(*self.blue.get(i/3).unwrap()),
					_ => unreachable!()
				}
			}
		}

		full
	}
}