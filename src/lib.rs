mod processor;
pub mod debayer;
pub mod image;

pub use processor::Processor;

use crate::image::{CFA, Metadata, Image, Sensor};

pub fn read_file(filename: &str) -> Image<Sensor, u16> {
	// Raw NEF data
	let nef_data = std::fs::read(filename).expect("Failed to read in raw file");
	let decoder = libraw::Processor::new();

	// Decode into raw sensor data
	let decoded = decoder.decode(&nef_data).expect("Failed to decode raw file");
	let sensor_data = (*decoded).to_vec();
	let sizes = decoded.sizes();

	let raw_size = sizes.raw_width as usize * sizes.raw_height as usize;
	let image_size = sizes.width as usize * sizes.height as usize;

	// TODO: Move to own function, call `extract_meaningful_image` maybe?
	if raw_size != image_size {
		let mut image = Vec::with_capacity(image_size);

		// FIXME: Assumes the extra data is to the right and/or bottom
		for row in 0..sizes.height as usize {
			let lower = row * sizes.raw_width as usize;
			let upper = lower + sizes.width as usize;

			image.extend_from_slice(&sensor_data[lower..upper]);
		}

		//FIXME: Assumes CFA:RGGB
		Image {
			kind: Sensor {},
			data: image,
			meta: Metadata::new(
				sizes.width as u32, 
				sizes.height as u32,
				CFA::RGGB,
				decoded.color()
			)
		}
	} else {
		//FIXME: Assumes CFA:RGGB
		Image {
			kind: Sensor {},
			data: sensor_data,
			meta: Metadata::new(
				sizes.raw_width as u32, 
				sizes.raw_height as u32,
				CFA::RGGB,
				decoded.color()
			)
		}
	}
}
