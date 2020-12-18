use crate::common::{Color, ComponentImage, RawImage};
use rand;

pub struct Debayer {}
impl Debayer {
	pub fn rgb(rimg: RawImage) -> ComponentImage<u16> {
		let capacity = rimg.meta.rgb_len();
		let mut rgb = vec![0; capacity];

		let mut i: usize = 0;
		for light in rimg.raw {
			match rimg.meta.color_at(i) {
				Color::Red => {
					rgb[i*3] = light;
					rgb[i*3 + 1] = 0;
					rgb[i*3 + 2] = 0;
				},
				Color::Green => {
					rgb[i*3] = 0;
					rgb[i*3 + 1] = light;
					rgb[i*3 + 2] = 0;
				},
				Color::Blue => {
					rgb[i*3] = 0;
					rgb[i*3 + 1] = 0;
					rgb[i*3 + 2] = light;
				}
			}

			i += 1;
		}

		ComponentImage {
			meta: rimg.meta,
			rgb
		}
	}
}

pub trait Interpolate<T: Copy> {
	fn interpolate(cimg: &mut ComponentImage<T>);
}

// Currently assumes RGGB bayering
pub struct NearestNeighbor {}

impl<T: Copy> Interpolate<T> for NearestNeighbor {
	fn interpolate(cimg: &mut ComponentImage<T>) {
		let pixel_count = (cimg.meta.width * cimg.meta.height) as usize;
		for pix in 0..pixel_count {
			match cimg.meta.color_at(pix) {
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
	fn get_component<T: Copy>(cimg: &ComponentImage<T>, color: Color, i: usize) -> T {
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