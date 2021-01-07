extern crate image as img;

mod color;
mod colors;
mod debayer;
mod image;

pub use color::Color;

use libraw::{ColorData, Processor};
use crate::image::{CFA, Metadata, RawImage};
use colors::Colors;
use debayer::{Debayer, Interpolate, NearestNeighbor};
use getopts::Options;
use std::time::Instant;
use std::fs::File;
use std::fs;
use std::str::FromStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use img::ImageBuffer;
use img::Rgb;

#[derive(Debug)]
struct CliArgs {
	pub is_dir: bool,
	pub in_file: String,
	pub out_file: String,
	pub ev: Option<f32>
}

impl CliArgs {
	pub fn new() -> Option<Self> {
		let args: Vec<String> = std::env::args().collect();
		let program = &args[0];
	
		let mut opts = Options::new();
		opts.reqopt("i", "ifile", "input file", "FILE");
		opts.reqopt("o", "ofile", "output file", "FILE");
		opts.optopt("b", "blackpoint", "blackpoint values\nleave empty to automatically determine", "BLACK");
		opts.optopt("w", "whitebalance", "white balance values\nleave empty to automatically determine", "WHITE");
		opts.optopt("e", "exposure", "Adjust exposure", "EV");
		let matches = match opts.parse(&args[1..]) {
			Ok(m) => m,
			Err(_) => {
				print_usage(program, opts);
				return None;
			}
		};
	
		let in_file = matches.opt_str("ifile").expect("How'd this happen?");
		let out_file = matches.opt_str("ofile").expect("How'd this happen?");
		let blackpoint = matches.opt_str("blackpoint").map(Self::values_parse::<u16>);
		let whitebalance = matches.opt_str("whitebalance").map(Self::values_parse::<f32>);
		let ev: Option<f32>  = matches.opt_str("exposure").map(|s| s.parse().expect("EV must be a float") );

		let in_meta = fs::metadata(&in_file).expect("Failed to get metadata for input file");
		let is_dir = if in_meta.is_dir() {
			println!("Input file is a directory. Processing every file in directory and using output as a directory name");
			true
		} else {
			false
		};

		Some(Self {
			is_dir,
			in_file,
			out_file,
			ev
		})
	}

	fn values_parse<T: FromStr>(args: String) -> Option<Vec<T>> {
		let mut arg_vec = vec![];
		for arg in args.split(',') {
			arg_vec.push(
				match arg.parse() {
					Ok(argok) => argok,
					Err(_) => return None
				}
			);
		}

		Some(arg_vec)
	}
}

fn print_usage(program: &str, opts: Options) {
	let brief = format!("Usage: {} FILE [options]", program);
	println!("{}", opts.usage(&brief));
}

fn main() {
	let cli = CliArgs::new().or_else(|| std::process::exit(1)).unwrap();

	println!("{:?}", cli);

	if cli.is_dir {
		process_directory(cli);
		return;
	}

	let (rawimg, colordata) = decode(&cli.in_file);
	get_rgb(&cli.out_file, rawimg, colordata, cli.ev);
}

fn process_directory(cli: CliArgs) {
	let threadpool = threadpool::Builder::new().build();

	let before = Instant::now();
	let contents = fs::read_dir(cli.in_file).expect("Failed to read input directory");
	let ev = cli.ev;

	for entry in contents {
		let entry = entry.expect("Failed reading a file");
		let mut filename = PathBuf::from(&entry.file_name());

		let in_file = filename.to_str().unwrap().to_owned();
		filename.set_extension("JPG");

		let mut out_file = PathBuf::from(&cli.out_file);
		out_file.push(filename);
		let out_file = out_file.to_str().unwrap().to_owned();

		if entry.metadata().expect("Failed getting a files metadata").is_file() {
			threadpool.execute(move || {
				let (rawimg, colordata) = decode(entry.path().to_str().unwrap());
				get_rgb(&out_file, rawimg, colordata, ev.clone());
				println!("Finished processing {}, saving as {}", in_file, out_file);
			})
		}
	}

	threadpool.join();
	println!("Finished processing directory in {} seconds", Instant::now().duration_since(before).as_secs());
}

fn decode(ifname: &str) -> (RawImage, ColorData) {
	println!("Decodin");
	// Raw NEF data
	let nef_data = std::fs::read(ifname).expect("Failed to read in raw file");
	let decoder = Processor::new();

	// Decode into raw sensor data
	let decoded = decoder.decode(&nef_data).expect("Failed to decode raw file");
	let sensor_data = (*decoded).to_vec();
	let sizes = decoded.sizes();

	(
		RawImage {
			meta: Metadata::new(CFA::RGGB, sizes.raw_width as u32, sizes.raw_height as u32),
			raw: sensor_data
		},
		decoded.color()
	)
}

fn get_rgb(oname: &str, mut rimg: RawImage, color: ColorData, ev: Option<f32>) {
	let get_time = |before: Instant, after: Instant| -> f32 {
		after.duration_since(before).as_millis() as f32 / 1000 as f32
	};

	let the_start_of_it_all = Instant::now();
	// Black Point
	let mut before = Instant::now();
	Colors::black(&mut rimg, color.black as u16, color.black as u16, color.black as u16);
	let mut after = Instant::now();
	println!("Black levels took {}s", get_time(before, after));

	// Poor mans white balance
	before = Instant::now();
	Colors::white(&mut rimg, color.cam_mul[0], color.cam_mul[1], color.cam_mul[2]);
	after = Instant::now();
	println!("White balance took {}s", get_time(before, after));

	if let Some(ev) = ev {
		before = Instant::now();
		Colors::adjust_exposure(&mut rimg, ev);
		after = Instant::now();
		println!("Adjusting exposure took {}s", get_time(before, after));
	}

	// Split the sensor data into its components
	before = Instant::now();
	let mut cimg = Debayer::rgb(rimg);
	after = Instant::now();
	println!("Spliting sensor data into components took {}s", get_time(before, after));

	// Nearest neighdoor debayering
	before = Instant::now();
	NearestNeighbor::interpolate(&mut cimg);
	after = Instant::now();
	println!("Nearet neighboor took {}s", get_time(before, after));

	let mut cimg = cimg.as_floats();

	// cam to sRGB
	before = Instant::now();
	Colors::to_sRGB(&mut cimg, &color);
	after = Instant::now();
	println!("cam to sRGB {}s", get_time(before, after));
	
	// sRGB gamma correction
	before = Instant::now();
	Colors::sRGB_gamma(&mut cimg);
	after = Instant::now();
	println!("sRGB gamma correction took {}s", get_time(before, after));

	let buf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(cimg.meta.width, cimg.meta.height, cimg.as_bytes()).unwrap();
	buf.save_with_format(oname, img::ImageFormat::Jpeg).expect("Failed to save jpg");

	/*let fout = File::create(oname).expect("Failed to create output file");
	let mut encoder = png::Encoder::new(fout, cimg.meta.width, cimg.meta.height);
	encoder.set_color(png::ColorType::RGB);
	encoder.set_depth(png::BitDepth::Eight);

	println!("Writing out fullres PNG image. This might take awhile...");
	before = Instant::now();
	let mut writer = encoder.write_header().expect("Failed to write PNG header");
	writer.write_image_data(&cimg.as_bytes()).expect("Failed to write image data");
	after = Instant::now();
	println!("Writing out PNG took {}s", get_time(before, after));

	let out_with_a_bang = Instant::now();
	let the_age_of_everthing = get_time(the_start_of_it_all, out_with_a_bang) as u32;
	println!("Processing took {}m{}s", the_age_of_everthing / 60, the_age_of_everthing % 60);*/
}