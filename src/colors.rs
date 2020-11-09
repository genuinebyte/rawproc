use crate::common::{Color, RawImage};

pub struct Colors {}
impl Colors {
	pub fn gamma(rimg: &mut RawImage) {
		for light in rimg.raw.iter_mut() {
			((*light as f32/ 4096.0).powf(1.0/2.2) * 4096.0) as u16; //TODO: 4096 allow different bitness
		}
	}
	
	pub fn white(rimg: &mut RawImage, red: f32, green: f32, blue: f32) {
		let mut i = 0;
		for light in rimg.raw.iter_mut() {
			match rimg.meta.color_at(i) {
				Color::Red => *light = (((*light as f32 / 4096.0) * red) * 4096.0) as u16,
				Color::Green => *light = (((*light as f32 / 4096.0) * green) * 4096.0) as u16,
				Color::Blue => *light = (((*light as f32 / 4096.0) * blue) * 4096.0) as u16,
			}
			i += 1;
		}
	}
}