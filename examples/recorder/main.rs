extern crate soundio;
extern crate crossbeam;
extern crate hound;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::BufWriter;
use std::fs::File;
use std::env;

struct WavRecorder {
	writer: hound::WavWriter<BufWriter<File>>,
}

impl WavRecorder {
	fn read_callback(&mut self, stream: &mut soundio::InStreamReader) {
		let frame_count_max = stream.frame_count_max();
		if let Err(e) = stream.begin_read(frame_count_max) {
			println!("Error reading from stream: {}", e);
			return;
		}

		for f in 0..stream.frame_count() {
			for c in 0..stream.channel_count() {
				match self.writer.write_sample(stream.sample_typed::<i16>(c, f)) {
					Ok(_) => {},
					Err(e) => println!("Error: {}", e),
				}
			}
		}
	}
}

// TODO: I need to implement Sync for WavPlayer so write_callback can be called from a different thread to finished().
// TODO: I also need to use interior mutability so that write_callback() doesn't mutably borrow self. But it *does* mutate self. The problem is that
// Rust doesn't know that it is safe to call the other functions while 

// Print sound soundio debug info and play back a sound.
fn record(filename: &str) -> Result<(), String> {
	// TODO: Probe which channels/sample rates are available.
	let channels = 1;
	let sample_rate = 44100;

	let spec = hound::WavSpec {
		channels: channels,
		sample_rate: sample_rate,
		bits_per_sample: 16,
		sample_format: hound::SampleFormat::Int,
	};

	// Try to open the output file.
	let writer = hound::WavWriter::create(filename, spec).map_err(|x| x.to_string())?;

	println!("Soundio version: {}", soundio::version_string());

	let mut ctx = soundio::Context::new("Recorder");
	ctx.connect()?;

	println!("Current backend: {:?}", ctx.current_backend());

	// We have to flush events so we can scan devices.
	println!("Flushing events.");
	ctx.flush_events();
	println!("Flushed");
	// I guess these are always signed little endian?
	let soundio_format = soundio::Format::S16LE;

	let default_layout = soundio::ChannelLayout::get_default(channels as _);
	println!("Default layout for {} channel(s): {:?}", channels, default_layout);

	let input_dev = ctx.default_input_device().map_err(|_| "Error getting default input device".to_string())?;

	println!("Default input device: {} {}", input_dev.name(), if input_dev.is_raw() { "raw" } else { "cooked" } );

	let mut recorder = WavRecorder {
		writer: writer,
	};

	println!("Opening default input stream");
	let mut input_stream = input_dev.open_instream(
		sample_rate as _,
		soundio_format,
		default_layout,
		2.0,
		|x| recorder.read_callback(x), // The trouble is this borrows &mut player, so I can't use it at all elsewhere. It's correct because player can be mutated. But I still want to read a value of it.
		None::<fn()>,
		None::<fn(soundio::Error)>,
	)?;

	println!("Starting stream");
	input_stream.start()?;

	let exit_loop = AtomicBool::new(false);
	let exit_loop_ref = &exit_loop;


	// Create a new thread scope...
	crossbeam::scope(|scope| {

		let ctx_ref = &ctx;

		// Start a new scoped thread to wait for sound events (and for the player to be finished).
		scope.spawn(move || {
			while exit_loop_ref.load(Ordering::Relaxed) == false {
				ctx_ref.wait_events();
			}
		});

		let stdin = io::stdin();
		let input = &mut String::new();

		input.clear();
		println!("Press enter to stop recording");
		let _ = stdin.read_line(input);
		exit_loop.store(true, Ordering::Relaxed);
		ctx.wakeup();
	});

	Ok(())
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		println!("Usage: {} <filename.wav>", args[0]);
		return;
	}

	match record(&args[1]) {
		Err(x) => println!("Error: {}", x),
		_ => {},
	}
}