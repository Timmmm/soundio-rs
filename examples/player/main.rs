extern crate soundio;
extern crate crossbeam;
extern crate hound;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::BufReader;
use std::fs::File;
use std::env;

struct WavPlayer {
	reader: hound::WavReader<BufReader<File>>,
	finished: bool,
}

impl WavPlayer {
	fn write_callback(&mut self, stream: &mut soundio::OutStreamWriter) {
		let frame_count_max = stream.frame_count_max();
		if let Err(e) = stream.begin_write(frame_count_max) {
			println!("Error writing to stream: {}", e);
			return;
		}

		let mut s = self.reader.samples();

		// TODO: Not hard-code the format here. In fact it would be nice if we could choose the format at run-time... Or get it from a soundio::Format at compile time?
		let was_finished = self.finished;

		for f in 0..stream.frame_count() {
    		for c in 0..stream.channel_count() {
				match s.next() {
					Some(x) => {
						stream.set_sample::<i16>(c, f, x.unwrap()); 
					},
					None => {
						stream.set_sample::<i16>(c, f, 0);
						self.finished = true;
					}
				}
				
			}
		}
		// TODO: When s.next() doesn't unwrap we need to stop playback by... maybe we can have a callback in WavPlayer when playback is finished.

		if self.finished != was_finished {
	//		stream.wakeup();
		}
	}

	fn finished(&self) -> bool {
		self.finished
	}
}
// TODO: I need to implement Sync for WavPlayer so write_callback can be called from a different thread to finished().
// TODO: I also need to use interior mutability so that write_callback() doesn't mutably borrow self. But it *does* mutate self. The problem is that
// Rust doesn't know that it is safe to call the other functions while 

// Print sound soundio debug info and play back a sound.
fn play(filename: &str) -> Result<(), String> {
	// Try to open the file.
	let reader = hound::WavReader::open(filename).map_err(|x| x.to_string())?;
	


	println!("Soundio version: {}", soundio::version_string());

	let mut ctx = soundio::Context::new();
	ctx.set_app_name("Player");
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

	let mut player = WavPlayer {
		reader: reader,
		finished: false,
	};

	println!("Opening default output stream");
	let mut output_stream = output_dev.open_outstream(
		sample_rate as _,
		soundio_format,
		default_layout,
		2.0,
		|x| player.write_callback(x), // The trouble is this borrows &mut player, so I can't use it at all elsewhere. It's correct because player can be mutated. But I still want to read a value of it.
		None::<fn()>,
		None::<fn(soundio::Error)>,
	)?;

	println!("Starting stream");
	output_stream.start()?;

	let exit_loop = AtomicBool::new(false);
	let exit_loop_ref = &exit_loop;


	// Create a new thread scope...
	crossbeam::scope(|scope| {


		let ctx_ref = &ctx;

		// Start a new scoped thread to wait for sound events (and for the player to be finished).
        scope.spawn(move || {
			while exit_loop_ref.load(Ordering::Relaxed) == false/* && !player.finished*/ {
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
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		println!("Usage: {} <filename.wav>", args[0]);
		return;
	}


	match play(&args[1]) {
		Err(x) => println!("Error: {}", x),
		_ => {},
	}
}