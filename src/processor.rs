use crate::image::{RgbImage, RawImage};
use crate::Color;
use libraw::ColorData;

pub struct Processor {}
impl Processor {
	pub fn black_levels(rimg: &mut RawImage, red: u16, green: u16, blue: u16) {
		let clamp = |light: u16, color: u16| {
			if light < color {
				0
			} else {
				light - color
			}
		};

		let mut i = 0;
		for light in rimg.raw.iter_mut() {
			match rimg.meta.color_at_index(i) {
				Color::Red => *light = clamp(*light, red),
				Color::Green => *light = clamp(*light, green),
				Color::Blue => *light = clamp(*light, blue)
			}
			i += 1;
		}
	}

	// https://photo.stackexchange.com/a/41936
	pub fn exposure(rimg: &mut RawImage, ev: f32) {
		for light in rimg.raw.iter_mut() {
			*light = ((*light as f32/ 4096.0) * 2f32.powf(ev) * 4096.0) as u16; //TODO: 4096 allow different bitness
		}
	}

	pub fn gamma(rimg: &mut RawImage, value: f32) {
		for light in rimg.raw.iter_mut() {
			*light = ((*light as f32/ 4096.0).powf(1.0/value) * 4096.0) as u16; //TODO: 4096 allow different bitness
		}
	}

	pub fn white_balance(rimg: &mut RawImage, red: f32, green: f32, blue: f32) {
		let mut i = 0;
		for light in rimg.raw.iter_mut() {
			match rimg.meta.color_at_index(i) {
				Color::Red => *light = (((*light as f32 / 4096.0) * red) * 4096.0) as u16,
				Color::Green => *light = (((*light as f32 / 4096.0) * green) * 4096.0) as u16,
				Color::Blue => *light = (((*light as f32 / 4096.0) * blue) * 4096.0) as u16,
			}
			i += 1;
		}
	}

	#[allow(non_snake_case)]
	pub fn to_sRGB(cimg: &mut RgbImage<f32>, colordata: &ColorData) {
		let mat = colordata.rgb_cam;
		for pix in cimg.pixel_range() {
			let (r, g, b) = (cimg.rgb[pix], cimg.rgb[pix+1], cimg.rgb[pix+2]);

			cimg.rgb[pix] = mat[0][0] * r + mat[0][1] * g + mat[0][2] * b;   // X
			cimg.rgb[pix+1] = mat[1][0] * r + mat[1][1] * g + mat[1][2] * b; // Y
			cimg.rgb[pix+2] = mat[2][0] * r + mat[2][1] * g + mat[2][2] * b; // Z
		}
	}

	#[allow(non_snake_case)]
	pub fn sRGB_gamma(cimg: &mut RgbImage<f32>) {
		for component in cimg.rgb.iter_mut() {
			// Value taken from Wikipedia page on sRGB
			// https://en.wikipedia.org/wiki/SRGB
			if *component <= 0.0031308 {
				*component *= 12.92;
			} else {
				*component = 1.055 * component.powf(1.0/2.4) - 0.055;
			}
		}
	}

	/*// XYZ to sRGB values taken from:
	// http://color.org/chardata/rgb/sRGB.pdf
	//FIXME: This doesn't work with the matrix got from to_xyz
	#[allow(non_snake_case)]
	pub fn xyz_to_linear_sRGB(cimg: &mut RgbImage<f32>) {
		// f32::clamp is in nightly, so we use this for now
		let clamp = |component: f32| -> f32 {
			if component < 0.0 {
				0.0
			} else if component > 1.0{
				1.0
			} else {
				component
			}
		};

		for pix in cimg.pixel_range() {
			let (x, y, z) = (cimg.rgb[pix], cimg.rgb[pix+1], cimg.rgb[pix+2]);

			cimg.rgb[pix] = clamp(3.2406255 * x + -1.537208 * y + -0.4986286 * z);   // R
			cimg.rgb[pix+1] = clamp(-0.9689307 * x + 1.8757561 * y + 0.0415175 * z); // G
			cimg.rgb[pix+2] = clamp(0.0557101 * x + -0.2040211 * y + 1.0569959 * z); // B
		}
	}

	//FIXME: Is this what's wrong with the xyz -> sRGB chain? Maybe because we whitebalance?
	pub fn to_xyz(cimg: &mut RgbImage<f32>, colordata: &ColorData) {
		let mat = colordata.cam_xyz;
		for pix in cimg.pixel_range() {
			let (r, g, b) = (cimg.rgb[pix], cimg.rgb[pix+1], cimg.rgb[pix+2]);

			cimg.rgb[pix] = mat[0][0] * r + mat[0][1] * g + mat[0][2] * b;   // X
			cimg.rgb[pix+1] = mat[1][0] * r + mat[1][1] * g + mat[1][2] * b; // Y
			cimg.rgb[pix+2] = mat[2][0] * r + mat[2][1] * g + mat[2][2] * b; // Z
		}
	}*/
}