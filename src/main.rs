mod colors;
mod common;
mod debayer;

use colors::Colors;
use common::{CFA, Metadata, RawImage};
use debayer::Debayer;
use getopts::Options;
use std::time::Instant;
use std::fs::File;

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} FILE [options]", program);
	println!("{}", opts.usage(&brief));
}

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let program = &args[0];

	let mut opts = Options::new();
	opts.reqopt("f", "file", "File to work with", "FILE");
	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => {
			print_usage(program, opts);
			return;
		}
	};

	let fname = matches.opt_str("file").expect("How'd this happen?");
	get_rgb(&fname);
}

fn get_rgb(fname: &str) -> Vec<u8> {
	
	// Raw NEF data
	let nef_data = std::fs::read(fname).expect("Read in");
	let decoder = libraw::Processor::new();

	// Decode into raw sensor data
	let mut before = Instant::now();
	let decoded = decoder.decode(&nef_data).expect("Failed to decode NEF");
	let mut after = Instant::now();
	let get_time = |b: Instant, a: Instant| a.duration_since(b).as_millis() as f32 / 1000f32;
	println!("NEF decode took {}s", get_time(before, after));

	let sensor_data = (*decoded).to_vec();
	println!("Sensor data is {} u16s long", sensor_data.len());

	let sizes = decoded.sizes();
	let color = decoded.color();

	let width = sizes.raw_width as u32;
	let height = sizes.raw_height as u32;
	println!("image is {}x{}", width, height);

	let mut rimg = RawImage {
		meta: Metadata::new(CFA::RGGB, width, height),
		raw: sensor_data
	};

	println!();
	color.debug();
	println!();

	// Black Point
	before = Instant::now();
	Colors::black(&mut rimg, color.black as u16, color.black as u16, color.black as u16);
	after = Instant::now();
	println!("Black levels took {}s", get_time(before, after));

	// Poor mans white balance
	before = Instant::now();
	Colors::white(&mut rimg, color.cam_mul[0], color.cam_mul[1] / 4.0, color.cam_mul[2]);
	after = Instant::now();
	println!("White balance took {}s", get_time(before, after));

	// Poor mans gamma crrection
	before = Instant::now();
	Colors::gamma(&mut rimg, 2.2);
	after = Instant::now();
	println!("Gamma correction took {}s", get_time(before, after));

	// Split the sensor data into its components
	before = Instant::now();
	let mut cimg = Debayer::rgb(rimg);
	after = Instant::now();
	println!("Spliting sensor data into components took {}s", get_time(before, after));

	// Nearest neighdoor debayering
	before = Instant::now();
	//unsafe { Debayer::nearest_neighboor(&mut cimg); }
	after = Instant::now();
	println!("Nearet neighboor took {}s", get_time(before, after));

	let fout = File::create(format!("{}.png", fname)).expect("Failed to create output file");
	let mut encoder = png::Encoder::new(fout, width, height);
	encoder.set_color(png::ColorType::RGB);
	encoder.set_depth(png::BitDepth::Eight);

	println!("Writing out fullres PNG image. This might take awhile...");
	before = Instant::now();
	let mut writer = encoder.write_header().expect("Failed to write PNG header");
	writer.write_image_data(&cimg.as_bytes()).expect("Failed to write image data");
	after = Instant::now();
	println!("Writing out PNG took {}s", get_time(before, after));

	vec![]
}