use crate::image::{Color, Image, Rgb, Sensor};
use rand;

pub struct Debayer {
	img: Image<Rgb, f32>
}

impl Debayer {
	pub fn new(mut rimg: Image<Sensor, f32>) -> Self {
		let sensor_len = rimg.data.len();
		rimg.data.resize(sensor_len * 3, 0.0);

		for i in (0..sensor_len).rev() {
			match rimg.meta.color_at_index(i) {
				Color::Red => {
					rimg.data[i*3] = rimg.data[i];
					rimg.data[i*3+1] = 0.0;
					rimg.data[i*3+2] = 0.0;
				},
				Color::Green => {
					rimg.data[i*3] = 0.0;
					rimg.data[i*3+1] = rimg.data[i];
					rimg.data[i*3+2] = 0.0;
				},
				Color::Blue => {
					rimg.data[i*3] = 0.0;
					rimg.data[i*3+1] = 0.0;
					rimg.data[i*3+2] = rimg.data[i];
				}
			}
		}

		Self {
			img: Image {
				kind: Rgb {},
				data: rimg.data,
				meta: rimg.meta
			}
		}
	}

	pub fn interpolate(mut self, interpolation: Interpolation) -> Image<Rgb, f32> {
		match interpolation {
			Interpolation::None => (),
			Interpolation::NearestNeighbor => NearestNeighbor::interpolate(&mut self.img)
		}

		self.img
	}
}

pub enum Interpolation {
	None,
	NearestNeighbor
}

// Currently assumes RGGB bayering
struct NearestNeighbor;
impl NearestNeighbor {
	fn interpolate(cimg: &mut Image<Rgb, f32>) {
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

	fn get_component(cimg: &Image<Rgb, f32>, color: Color, i: usize) -> f32 {
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