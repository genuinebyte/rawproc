use crate::image::{Attribute ,Color, Image, Sensor, Rgb, Hsv};
use std::cmp::min;

pub struct Processor {}
impl Processor {
	fn f32clamp(value: f32, min: f32, max: f32) -> f32 {
		max.min(min.max(value))
	}

	fn normalclamp(value: f32) -> f32{
		Self::f32clamp(value, 0.0, 1.0)
	}

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
			*light = Self::normalclamp(*light * 2f32.powf(ev));
		}
	}

	pub fn gamma(rimg: &mut Image<Sensor, f32>, value: f32) {
		for light in rimg.data.iter_mut() {
			*light = Self::normalclamp((*light).powf(1.0/value));
		}
	}

	pub fn white_balance(rimg: &mut Image<Sensor, f32>, red: f32, green: f32, blue: f32) {
		let mut i = 0;
		for light in rimg.data.iter_mut() {
			match rimg.meta.color_at_index(i) {
				Color::Red => *light = Self::normalclamp(*light * red),
				Color::Green => *light = Self::normalclamp(*light * green),
				Color::Blue => *light = Self::normalclamp(*light * blue),
			}
			i += 1;
		}
	}

	// https://math.stackexchange.com/a/906280
	pub fn brightness(cimg: &mut Image<Hsv, f32>, value: f32) {
		for comp in cimg.data.iter_mut() {
			*comp = Self::normalclamp(*comp + value);
		}
	}

	// https://math.stackexchange.com/a/906280
	pub fn contrast(cimg: &mut Image<Rgb, u8>, value: f32) {
		for comp in cimg.data.iter_mut() {
			*comp = 0f32.max(255f32.min(value * (*comp as f32 - 128.0) + 128.0)) as u8;
		}
	}

	pub fn saturation(img: &mut Image<Hsv, f32>, scalar: f32) {
		for saturation in img.component_iter_mut(Attribute::Saturation) {
			*saturation =  Self::normalclamp(*saturation * scalar);
		}
	}

	#[allow(non_snake_case)]
	pub fn to_sRGB(cimg: &mut Image<Rgb, f32>) {
		let mat = cimg.meta.colordata.rgb_cam;
		for pix in cimg.pixel_index_range() {
			let (r, g, b) = (cimg.data[pix], cimg.data[pix+1], cimg.data[pix+2]);

			cimg.data[pix] = Self::normalclamp(mat[0][0] * r + mat[0][1] * g + mat[0][2] * b);   // X
			cimg.data[pix+1] = Self::normalclamp(mat[1][0] * r + mat[1][1] * g + mat[1][2] * b); // Y
			cimg.data[pix+2] = Self::normalclamp(mat[2][0] * r + mat[2][1] * g + mat[2][2] * b); // Z
		}
	}

	#[allow(non_snake_case)]
	pub fn sRGB_gamma(cimg: &mut Image<Rgb, f32>) {
		for component in cimg.data.iter_mut() {
			// Value taken from Wikipedia page on sRGB
			// https://en.wikipedia.org/wiki/SRGB
			if *component <= 0.0031308 {
				*component = Self::f32clamp(*component * 12.92, 0.0, 1.0);
			} else {
				*component = Self::f32clamp(1.055 * component.powf(1.0/2.4) - 0.055, 0.0, 1.0);
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

		/*if hue < 0.0 {
			panic!("Hue negative: {}\nRGB ({},{},{})\nValue `{}`, Chroma `{}`", hue, r, g, b, value, chroma)
		}*/

		let value_saturation = if value == 0.0 {				
			0.0
		} else {
			chroma / value
		};

		/* Rotate the color wheel counter clockwise to the negative location
               |       Keep the wheel in place and remove any full rotations
		  _____V____ _____V____ 
		 |          |          |*/
		( (hue + 360.0) % 360.0, value_saturation, value)
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
			panic!("This shouldn't be able to happen! HSV ({},{},{})",
			hue, saturation, value);
		}
	}
}

#[cfg(test)]
mod cfa_tets {
	use super::*;

	// Simple colors. Maxed Red, Green, and Blue
	#[test]
	fn rgb_to_hsv_simple() {
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

	// More complex colors (not just 0 and 1)
	#[test]
	fn rgb_to_hsv() {
		fn assert_close(a: (f32, f32, f32), b: (f32, f32, f32)) {
			let tolerance = 3.9e-3; // 1 step in 8bit color
			if (a.0 - b.0).abs() > tolerance || 
			   (a.1 - b.1).abs() > tolerance ||
			   (a.2 - b.2).abs() > tolerance 
			{
				panic!(
					"assertion failed: `(left ~ right)`\n\
					\tLeft: `{:?}`,\n\
					\tRight:`{:?}`\n\
					Deviation allowed from left (tolerance is {}):\n\
					\tMax: `{:?}`\n\
					\tMin: `{:?}`\n\
					Actual Deviation: `{:?}`",
					a,
					b,
					tolerance,
					(a.0 - tolerance, a.1 - tolerance, 1.2 - tolerance),
					(a.0 + tolerance, a.1 + tolerance, 1.2 + tolerance),
					(a.0 - b.0, a.1 - b.1, a.2 - b.2)
				)
			}
		}

		// Darkish cyan
		assert_close(
			Processor::pixel_rgb_to_hsv(0.438, 0.875, 0.875),
			(180.0, 0.5, 0.875)
		);

		// Pinkish black
		assert_close(
			Processor::pixel_rgb_to_hsv(0.25, 0.125, 0.125),
			(0.0, 0.5, 0.25)
		);
	}

	// Simple colors. Maxed Red, Green, and Blue
	#[test]
	fn hsv_to_rgb_simple() {
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