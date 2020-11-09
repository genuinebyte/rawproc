use crate::common::{Color, ComponentImage};

pub struct Debayer {}
impl Debayer {
	pub fn rgb(raw: &[u16], cimg: &mut ComponentImage) {
		let capacity = cimg.meta.rgb_len();
		
		if cimg.rgb.len() != capacity {
			panic!("Expected buffer of size {} but got {}", capacity, cimg.rgb.len());
		}

		let mut i: usize = 0;
		for light in raw {
			let light = *light;
			match cimg.meta.color_at(i as u32) {
				Color::Red => {
					cimg.rgb[i*3] = light;
					cimg.rgb[i*3 + 1] = 0;
					cimg.rgb[i*3 + 2] = 0;
				},
				Color::Green => {
					cimg.rgb[i*3] = 0;
					cimg.rgb[i*3 + 1] = light;
					cimg.rgb[i*3 + 2] = 0;
				},
				Color::Blue => {
					cimg.rgb[i*3] = 0;
					cimg.rgb[i*3 + 1] = 0;
					cimg.rgb[i*3 + 2] = light;
				}
			}

			i += 1;
		}
	}

	fn get_pixel(cimg: &ComponentImage, color: Color, x: u32, y: u32) -> u16 {
		if !cimg.meta.are_coords_valid(x, y) {
			panic!(format!("Tried to access a pixel out of the images bounds: {}, {}", x, y));
		}

		let index = ((y * cimg.meta.width + x) * 3) as usize;
		unsafe {
			match color {
				Color::Red => {
					*cimg.rgb.get_unchecked(index)
				},
				Color::Green => {
					*cimg.rgb.get_unchecked(index+1)
				},
				Color::Blue => {
					*cimg.rgb.get_unchecked(index+2)
				}
			}
		}
	}

	fn set_pixel(cimg: &mut ComponentImage, color: Color, value: u16, x: u32, y: u32) {
		if !cimg.meta.are_coords_valid(x, y) {
			panic!(format!("Tried to set a pixel out of the images bounds: {}, {}", x, y));
		}

		let index = ((y * cimg.meta.width + x) * 3) as usize;
		unsafe {
			match color {
				Color::Red => {
					*cimg.rgb.get_unchecked_mut(index) = value;
				},
				Color::Green => {
					*cimg.rgb.get_unchecked_mut(index+1) = value;
				},
				Color::Blue => {
					*cimg.rgb.get_unchecked_mut(index+2) = value;
				}
			}
		}
	}

	//FIXME: This isn't proper nearest neighboor at all, but uh, it's good enough for now?
	pub unsafe fn nearest_neighboor(cimg: &mut ComponentImage) {
		let mut red = *cimg.rgb.get_unchecked(0);
		let mut green = *cimg.rgb.get_unchecked(1);
		let mut blue = *cimg.rgb.get_unchecked(2);

		for x in 0..cimg.meta.width {
			for y in 0..cimg.meta.height {
				match cimg.meta.color_at_xy(x, y) {
					Color::Red => {
						red = Self::get_pixel(cimg, Color::Red, x, y);
						Self::set_pixel(cimg, Color::Green, green, x, y);
						Self::set_pixel(cimg, Color::Blue, blue, x, y);
					},
					Color::Green => {
						Self::set_pixel(cimg, Color::Red, red, x, y);
						green = Self::get_pixel(cimg, Color::Green, x, y);
						Self::set_pixel(cimg, Color::Blue, blue, x, y);
					},
					Color::Blue => {
						Self::set_pixel(cimg, Color::Red, red, x, y);
						Self::set_pixel(cimg, Color::Green, green, x, y);
						blue = Self::get_pixel(cimg, Color::Blue, x, y);
					},
				}
			}
		}
	}
}