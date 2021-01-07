mod color;
mod processor;
pub mod debayer;
pub mod image;

pub use color::Color;
pub use processor::Processor;

use crate::image::{CFA, Metadata, RawImage};

pub fn read_file(filename: &str) -> RawImage {
	// Raw NEF data
	let nef_data = std::fs::read(filename).expect("Failed to read in raw file");
	let decoder = libraw::Processor::new();

	// Decode into raw sensor data
	let decoded = decoder.decode(&nef_data).expect("Failed to decode raw file");
	let sensor_data = (*decoded).to_vec();
	let sizes = decoded.sizes();

	RawImage {
		meta: Metadata::new(
			sizes.raw_width as u32, 
			sizes.raw_height as u32,
			CFA::RGGB,
			decoded.color()
		),
		raw: sensor_data
	}
}
