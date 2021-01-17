use crate::image::{Image, Rgb, Sensor};
use crate::Color;
use rand;
use num_traits::Num;

pub struct Debayer {}
impl Debayer {
	pub fn rgb<T: Num + Copy>(rimg: Image<Sensor, T>) -> Image<Rgb, T> {
		let capacity = rimg.meta.pixels() * 3;
		let mut rgb = vec![T::zero(); capacity];

		let mut i: usize = 0;
		for light in rimg.data {
			match rimg.meta.color_at_index(i) {
				Color::Red => {
					rgb[i*3] = light;
					rgb[i*3 + 1] = T::zero();
					rgb[i*3 + 2] = T::zero();
				},
				Color::Green => {
					rgb[i*3] = T::zero();
					rgb[i*3 + 1] = light;
					rgb[i*3 + 2] = T::zero();
				},
				Color::Blue => {
					rgb[i*3] = T::zero();
					rgb[i*3 + 1] = T::zero();
					rgb[i*3 + 2] = light;
				}
			}

			i += 1;
		}

		Image {
			kind: Rgb {},
			data: rgb,
			meta: rimg.meta
		}
	}
}

pub trait Interpolate<T: Num + Copy> {
	fn interpolate(cimg: &mut Image<Rgb, T>);
}

// Currently assumes RGGB bayering
pub struct NearestNeighbor {}

impl<T: Num + Copy> Interpolate<T> for NearestNeighbor {
	fn interpolate(cimg: &mut Image<Rgb, T>) {
		for pix in cimg.pixel_range() {
			match cimg.meta.color_at_index(pix) {
				Color::Red => {
					cimg.set_component(pix, Color::Green, Self::get_component(cimg, Color::Green, pix));
					cimg.set_component(pix, Color::Blue, Self::get_component(cimg, Color::Blue, pix));
				},
				Color::Green => {
					cimg.set_component(pix, Color::Red, Self::get_component(cimg, Color::Red, pix));
					cimg.set_component(pix, Color::Blue, Self::get_component(cimg, Color::Blue, pix));
				},
				Color::Blue => {
					cimg.set_component(pix, Color::Red, Self::get_component(cimg, Color::Red, pix));
					cimg.set_component(pix, Color::Green, Self::get_component(cimg, Color::Green, pix));
				}
			}
		}
	}
}

impl NearestNeighbor {
	fn get_component<T: Num + Copy>(cimg: &Image<Rgb, T>, color: Color, i: usize) -> T {
		let (x, y) = cimg.meta.itoxy(i);

		let top_color = if y == 0 {
			// There is no top pixel
			false
		} else if y == cimg.meta.height-1 {
			// There is no bottom pixel
			true
		} else {
			// Use a random top/bottom
			rand::random()
		};

		let left_color = if x == 0 {
			// There is no left color
			false
		} else if x == cimg.meta.width-1 {
			// There is no right color
			true
		} else {
			// Use a random left/right
			rand::random()
		};

		let color_x = if left_color {
			x - 1
		} else {
			x + 1
		};

		let color_y = if top_color {
			y - 1
		} else {
			y + 1
		};

		let current_color = cimg.meta.color_at_xy(x, y);

		match current_color {
			Color::Red => match color {
				Color::Red => {
					cimg.component(x, y, current_color)
				},
				Color::Green => {
					if rand::random() {
						cimg.component(x, color_y, color)
					} else {
						cimg.component(color_x, y, color)
					}
				},
				Color::Blue => {
					cimg.component(color_x, color_y, color)
				}
			},
			Color::Green => {
				let x_even = x%2 == 0;

				match color {
					Color::Red => if x_even {
						cimg.component(x, color_y, color)
					} else {
						cimg.component(color_x, y, color)
					},
					Color::Green => {
						cimg.component(x, y, current_color)
					},
					Color::Blue => if x_even {
						cimg.component(color_x, y, color)
					} else {
						cimg.component(x, color_y, color)
					}
				}
			},
			Color::Blue => match color {
				Color::Red => {
					cimg.component(color_x, color_y, color)
				},
				Color::Green => {
					if rand::random() {
						cimg.component(x, color_y, color)
					} else {
						cimg.component(color_x, y, color)
					}
				},
				Color::Blue => {
					cimg.component(x, y, current_color)
				}
			}
		}
	}
}