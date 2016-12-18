extern crate soundio;
extern crate rand;
extern crate crossbeam;
extern crate hound;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::BufReader;
use std::fs::File;

struct WavPlayer {
	reader: hound::WavReader<BufReader<File>>,
}

impl WavPlayer {
	fn write_callback(&mut self, stream: &mut soundio::OutStreamWriter) {
		let frame_count_max = stream.frame_count_max();
		if let Err(e) = stream.begin_write(frame_count_max) {
			println!("Error writing to stream: {}", e);
			return;
		}

		let mut s = self.reader.samples();
		
		for c in 0..stream.channel_count() {
			for f in 0..stream.frame_count() {
				stream.set_sample::<i16>(c, f, s.next().unwrap_or(Ok(0)).unwrap()); // TODO: Not hard-code the format here.
			}
		}
	}
}

// Print sound soundio debug info and play back a sound.
fn play(filename: &str) -> Result<(), String> {
	// Try to open the file.
	let reader = hound::WavReader::open(filename).map_err(|x| x.to_string())?;
	


	println!("Soundio version: {}", soundio::version_string());

	let mut ctx = soundio::Context::new("Wav Player");
	ctx.connect()?;

	println!("Current backend: {:?}", ctx.current_backend());

	// We have to flush events so we can scan devices.
	println!("Flushing events.");
	ctx.flush_events();
	println!("Flushed");


	let channels = reader.spec().channels;
	let sample_rate = reader.spec().sample_rate;
	let int_or_float = reader.spec().sample_format;
	let bits_per_sample = reader.spec().bits_per_sample;

	// I guess these are always signed little endian?
	let soundio_format = match int_or_float {
		hound::SampleFormat::Int => match bits_per_sample {
				8 => soundio::Format::S8,
				16 => soundio::Format::S16LE,
				24 => soundio::Format::S24LE,
				32 => soundio::Format::S32LE,
				_ => return Err(format!("Unknown bit depth: {}", bits_per_sample)),
			},

		hound::SampleFormat::Float => match bits_per_sample {
				32 => soundio::Format::Float32LE,
				64 => soundio::Format::Float64LE,
				_ => return Err(format!("Unknown bit depth: {}", bits_per_sample)),
			},
	};

	let default_layout = soundio::ChannelLayout::get_default(channels as _);
	println!("Default layout for {} channel(s): {:?}", channels, default_layout);

	let output_dev = ctx.default_output_device().map_err(|_| "Error getting default output device".to_string())?;

	println!("Default output device: {} {}", output_dev.name(), if output_dev.is_raw() { "raw" } else { "cooked" } );

	// What I want to do is something like this:

	let mut player = WavPlayer {
		reader: reader,
	};

	println!("Opening default output stream");
	let mut output_stream = output_dev.open_outstream(
		sample_rate as _,
		soundio_format,
		default_layout,
		2.0,
		move |x| player.write_callback(x),
		None::<fn()>,
		None::<fn(soundio::Error)>,
	)?;

	println!("Starting stream");
	output_stream.start()?;

	let exit_loop = AtomicBool::new(false);
	let exit_loop_ref = &exit_loop;

	// Run the loop in a new thread.

	crossbeam::scope(|scope| {

		let ctx_ref = &ctx;

        scope.spawn(move || {
			while exit_loop_ref.load(Ordering::Relaxed) != true {
				ctx_ref.wait_events();
			}
		});

		let stdin = io::stdin();
		let input = &mut String::new();

		input.clear();
		println!("Press enter to stop playback");
		let _ = stdin.read_line(input);
		exit_loop.store(true, Ordering::Relaxed);
		ctx.wakeup();
	});
	// Wait for key presses.
	Ok(())
}

fn main() {
	match play("test.wav") {
		Err(x) => println!("Error: {}", x),
		_ => {},
	}
}