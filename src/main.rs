mod debayer;

use debayer::{Debayer, CFA};
use getopts::Options;
use image::ColorType;
use image::png::PngEncoder;
use rawloader::{self, RawImageData};
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
	let mut before = Instant::now();
	let image = rawloader::decode_file(fname).expect("Failed to decode file!");
	let mut after = Instant::now();
	let get_time = |b: Instant, a: Instant| a.duration_since(b).as_millis() as f32 / 1000f32;

	println!("Decoded {} raw in {}s\nImage is {}x{}", fname, get_time(before, after), image.width, image.height);

	let sensor_data = match image.data {
		RawImageData::Integer(vec) => vec,
		_ => panic!("Dan't understand float image data!")
	};

	before = Instant::now();
	let mut debayer = Debayer::new(sensor_data, CFA::RGGB, image.width, image.height);
	after = Instant::now();

	println!("Spliting sensor data into components took {}s", get_time(before, after));

	before = Instant::now();
	debayer.nearest_neighboor();
	after = Instant::now();

	println!("Nearet neighboor took {}s", get_time(before, after));

	/*before = Instant::now();

	let fred = File::create(format!("{}.red.png", fname)).unwrap();
	unsafe {
		let red_data = std::slice::from_raw_parts(debayer.get_red().as_ptr() as *const u8, image.width * image.height * 2);
		PngEncoder::new(fred).encode(red_data, image.width as u32, image.height as u32, ColorType::L16).expect("Failed to encode red png");
	}

	after = Instant::now();

	println!("Took {}s to create the red PNG", get_time(before, after));*/

	before = Instant::now();

	let fred = File::create(format!("{}.green.png", fname)).unwrap();
	unsafe {
		let fullcolor = debayer.get_green();
		println!("After fullcolor");
		let green_data = std::slice::from_raw_parts(fullcolor.as_ptr() as *const u8, image.width * image.height * 2);
		PngEncoder::new(fred).encode(green_data, image.width as u32, image.height as u32, ColorType::L16).expect("Failed to encode full-color png");
	}

	after = Instant::now();

	println!("Took {}s to create the full-color PNG", get_time(before, after));

	vec![]
}