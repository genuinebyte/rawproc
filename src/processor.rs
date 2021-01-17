use crate::image::{Image, Sensor, Rgb, Hsv};
use crate::Color;
use std::cmp::min;

pub struct Processor {}
impl Processor {
	pub fn black_levels(rimg: &mut Image<Sensor, u16>, red: u16, green: u16, blue: u16) {
		let clamp = |light: u16, color: u16| {
			if light < color {
				0
			} else {
				light - color
			}
		};

		let mut i = 0;
		for light in rimg.data.iter_mut() {
			match rimg.meta.color_at_index(i) {
				Color::Red => *light = clamp(*light, red),
				Color::Green => *light = clamp(*light, green),
				Color::Blue => *light = clamp(*light, blue)
			}
			i += 1;
		}
	}

	// https://photo.stackexchange.com/a/41936
	pub fn exposure(rimg: &mut Image<Sensor, f32>, ev: f32) {
		for light in rimg.data.iter_mut() {
			*light = *light * 2f32.powf(ev);
		}
	}

	pub fn gamma(rimg: &mut Image<Sensor, f32>, value: f32) {
		for light in rimg.data.iter_mut() {
			*light = (*light).powf(1.0/value);
		}
	}

	pub fn white_balance(rimg: &mut Image<Sensor, f32>, red: f32, green: f32, blue: f32) {
		let mut i = 0;
		for light in rimg.data.iter_mut() {
			match rimg.meta.color_at_index(i) {
				Color::Red => *light = *light * red,
				Color::Green => *light = *light * green,
				Color::Blue => *light = *light * blue,
			}
			i += 1;
		}
	}

	// https://math.stackexchange.com/a/906280
	pub fn brightness(cimg: &mut Image<Rgb, u8>, value: u8) {
		for comp in cimg.data.iter_mut() {
			*comp = min((*comp as u16) + value as u16, 256u16) as u8;
		}
	}

	// https://math.stackexchange.com/a/906280
	pub fn contrast(cimg: &mut Image<Rgb, u8>, value: f32) {
		for comp in cimg.data.iter_mut() {
			let adjusted = 0f32.max(value * (*comp as f32 - 128.0) + 128.0);
			*comp = min(adjusted as u16, 256u16) as u8;
		}
	}

	#[allow(non_snake_case)]
	pub fn to_sRGB(cimg: &mut Image<Rgb, f32>) {
		let mat = cimg.meta.colordata.rgb_cam;
		for pix in cimg.pixel_index_range() {
			let (r, g, b) = (cimg.data[pix], cimg.data[pix+1], cimg.data[pix+2]);

			cimg.data[pix] = mat[0][0] * r + mat[0][1] * g + mat[0][2] * b;   // X
			cimg.data[pix+1] = mat[1][0] * r + mat[1][1] * g + mat[1][2] * b; // Y
			cimg.data[pix+2] = mat[2][0] * r + mat[2][1] * g + mat[2][2] * b; // Z
		}
	}

	#[allow(non_snake_case)]
	pub fn sRGB_gamma(cimg: &mut Image<Rgb, f32>) {
		for component in cimg.data.iter_mut() {
			// Value taken from Wikipedia page on sRGB
			// https://en.wikipedia.org/wiki/SRGB
			if *component <= 0.0031308 {
				*component *= 12.92;
			} else {
				*component = 1.055 * component.powf(1.0/2.4) - 0.055;
			}
		}
	}

	// https://en.wikipedia.org/wiki/HSL_and_HSV#From_RGB
	pub fn rgb_to_hsv(mut rgb: Image<Rgb, f32>) -> Image<Hsv, f32> {
		for pix in rgb.pixel_index_range() {
			let (h, s, v) = Processor::pixel_rgb_to_hsv(rgb.data[pix], rgb.data[pix+1], rgb.data[pix+2]);

			rgb.data[pix] = h;
			rgb.data[pix+1] = s;
			rgb.data[pix+2] = v;
		}

		Image {
			data: rgb.data,
			kind: Hsv {},
			meta: rgb.meta
		}
	}

	fn pixel_rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
		let value = r.max(g.max(b));
		let x_min = r.min(g.min(b));
		let chroma = value - x_min;

		let hue = if chroma == 0.0 {
			0.0
		} else if value == r {
			60.0 * ((g - b) / chroma)
		} else if value == g {
			60.0 * (2.0 + (b - r) / chroma)
		} else if value == b {
			60.0 * (4.0 + (r - g) / chroma)
		} else {
			unreachable!()
		};

		let value_saturation = if value == 0.0 {				
			0.0
		} else {
			chroma / value
		};

		(hue, value_saturation, value)
	}

	// https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB
	pub fn hsv_to_rgb(mut hsv: Image<Hsv, f32>) -> Image<Rgb, f32> {
		for pix in hsv.pixel_index_range() {
			let (r, g, b) = Processor::pixel_hsv_to_rgb(hsv.data[pix], hsv.data[pix+1], hsv.data[pix+2]);

			hsv.data[pix] = r;
			hsv.data[pix+1] = g;
			hsv.data[pix+2] = b;
		}

		Image {
			data: hsv.data,
			kind: Rgb {},
			meta: hsv.meta
		}
	}

	fn pixel_hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (f32, f32, f32) {
		let chroma = value * saturation;
		let hue_prime = hue / 60.0;
		let x = chroma * (1.0 - (hue_prime%2.0 - 1.0).abs());

		let m = value - chroma;
		let cm = chroma + m;
		let xm = x + m;

		if 0.0 <= hue_prime && hue_prime <= 1.0 {
			(cm, xm, m)
		} else if 1.0 < hue_prime && hue_prime <= 2.0 {
			(xm, cm, m)
		} else if 2.0 < hue_prime && hue_prime <= 3.0 {
			(m, cm, xm)
		} else if 3.0 < hue_prime && hue_prime <= 4.0 {
			(m, xm, cm)
		} else if 4.0 < hue_prime && hue_prime <= 5.0 {
			(xm, m, cm)
		} else if 5.0 < hue_prime && hue_prime <= 6.0 {
			(cm, m, xm)
		} else {
			unreachable!()
		}
	}
}

#[cfg(test)]
mod cfa_tets {
	use super::*;

	#[test]
	fn rgb_to_hsv() {
		// White. No saturation (no "colorfullness") and all value (all light emmitted, kind of)
		let (_h, s, v) = Processor::pixel_rgb_to_hsv(1.0, 1.0, 1.0);
		assert_eq!((s, v), (0.0, 1.0));

		// Full Red. Hue at 0 degrees, all colorfullness and all lit
		assert_eq!(
			Processor::pixel_rgb_to_hsv(1.0, 0.0, 0.0),
			(0.0, 1.0, 1.0)
		);

		// Full Green
		assert_eq!(
			Processor::pixel_rgb_to_hsv(0.0, 1.0, 0.0),
			(120.0, 1.0, 1.0)
		);

		// Full Blue
		assert_eq!(
			Processor::pixel_rgb_to_hsv(0.0, 0.0, 1.0),
			(240.0, 1.0, 1.0)
		);
	}

	#[test]
	fn hsv_to_rgb() {
		// White. Every color maxed
		assert_eq!(
			Processor::pixel_hsv_to_rgb(0.0, 0.0, 1.0),
			(1.0, 1.0, 1.0)
		);

		assert_eq!(
			Processor::pixel_hsv_to_rgb(0.0, 1.0, 1.0),
			(1.0, 0.0, 0.0)
		);

		assert_eq!(
			Processor::pixel_hsv_to_rgb(120.0, 1.0, 1.0),
			(0.0, 1.0, 0.0)
		);

		assert_eq!(
			Processor::pixel_hsv_to_rgb(240.0, 1.0, 1.0),
			(0.0, 0.0, 1.0)
		);
	}
}