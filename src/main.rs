extern crate soundio;
extern crate rand;

use std::f64::consts::PI;
use std::thread;
use std::io;

struct SineWavePlayer {
	phase: f64, // Phase is updated each time the write callback is called.
	frequency: f64,
	amplitude: f64,
}

impl SineWavePlayer {
	fn write_callback(&mut self, stream: &mut soundio::OutStreamWriter) {
		println!("my_write_callback called! Min/max frames: {}, {}; latency: {}", stream.frame_count_min(), stream.frame_count_max(), stream.get_latency().unwrap_or(-1.0));

		let frame_count_max = stream.frame_count_max();
		let mut channel_areas = match stream.begin_write(frame_count_max) {
			Ok(x) => x,
			Err(e) => {
				println!("Error writing to stream: {}", e);
				return;
			}
		};

		let phase_step = self.frequency / stream.sample_rate() as f64 * 2.0 * PI;

		for c in 0..channel_areas.channel_count() {
			for f in 0..channel_areas.frame_count() {
				channel_areas.set_sample::<f32>(c, f, (self.phase.sin() * self.amplitude) as f32);
				self.phase += phase_step;
			}
		}
	}
}
/*
fn my_write_callback(stream: &mut soundio::StreamWriter) {
	println!("my_write_callback called! Min/max frames: {}, {}; latency: {}", stream.frame_count_min(), stream.frame_count_max(), stream.get_latency().unwrap_or(-1.0));
	let mut channel_areas = match stream.begin_write(stream.frame_count_max()) {
		Ok(x) => x,
		Err(e) => {
			println!("Error writing to stream: {}", e);
			return;
		}
	};

	for c in 0..channel_areas.channel_count() {
		for f in 0..channel_areas.frame_count() {
			channel_areas.set_sample(c, f, rand::random::<f32>());
		}
	}

	// let mut channel_left = channel_areas.get_slice(0);
	// for i in 0..channel_left.len() {
	// 	channel_left[i] = 0;
	// }
}*/

// Print sound soundio debug info and play back a sound.
fn run() -> Result<(), String> {

	println!("Soundio version: {}", soundio::version_string());

	let (major, minor, patch) = soundio::version();

	println!("Major version: {}, minor version: {}, patch version: {}", major, minor, patch);

	let backend_list = [
		soundio::Backend::Jack,
		soundio::Backend::PulseAudio,
		soundio::Backend::Alsa,
		soundio::Backend::CoreAudio,
		soundio::Backend::Wasapi,
		soundio::Backend::Dummy,
	];

	for &backend in backend_list.iter() {
		println!("Backend {} available? {}", backend, soundio::have_backend(backend));
	} 

	println!("InitAudioBackend error: {}", soundio::Error::InitAudioBackend);

	let mut ctx = soundio::Context::new("Test App");

	println!("Available backends: {:?}", ctx.available_backends());

	ctx.connect()?;

	println!("Current backend: {:?}", ctx.current_backend());

	// We have to flush events so we can scan devices.
	println!("Flushing events.");
	ctx.flush_events();
	println!("Flushed");

	// Builtin and default layouts.

	let builtin_layouts = soundio::ChannelLayout::get_builtin();
	for layout in builtin_layouts {
		println!("Builtin layout: {:?}", layout);
	}

	let default_mono_layout = soundio::ChannelLayout::get_default(1);
	println!("Default mono layout: {:?}", default_mono_layout);
	let default_stereo_layout = soundio::ChannelLayout::get_default(2);
	println!("Default stereo layout: {:?}", default_stereo_layout);


	println!("Input device count: {}", ctx.input_device_count().unwrap_or(0));
	println!("Output device count: {}", ctx.output_device_count().unwrap_or(0));

	let output_devices = ctx.output_devices().map_err(|_| "Error getting output devices".to_string())?;
	let input_devices = ctx.input_devices().map_err(|_| "Error getting input devices".to_string())?;

	for dev in output_devices {
		println!("Output device: {} {}", dev.name(), if dev.is_raw() { "raw" } else { "cooked" } );
	}

	for dev in input_devices {
		println!("Input device: {} {}", dev.name(), if dev.is_raw() { "raw" } else { "cooked" } );
	}

	let output_dev = ctx.default_output_device().map_err(|_| "Error getting default output device".to_string())?;

	println!("Default output device: {} {}", output_dev.name(), if output_dev.is_raw() { "raw" } else { "cooked" } );

	// What I want to do is something like this:

	let mut sine = SineWavePlayer {
		phase: 0.0,
		amplitude: 0.5,
		frequency: 400.0,
	};

	println!("Opening default output stream");
	let mut output_stream = output_dev.open_outstream(
		48000,
		soundio::Format::Float32LE,
		soundio::ChannelLayout::get_default(2),
		move |x| sine.write_callback(x),
	)?;

	println!("Starting stream");
	output_stream.start()?;

	// Run the loop in a new thread.
	let child = thread::spawn(move || {
		while exit_cv == 0 {
			println!("wait_events");
			ctx.wait_events();
			println!("waited");
		}
	});

	// Wait for key presses.

	let mut stdin = io::stdin();
	let input = &mut String::new();

	input.clear();
	stdin.read_line(input);
	exit_cv = 1;
	ctx.wake_up();

	child.join();
}

fn main() {
	match run() {
		Err(x) => println!("Error: {}", x),
		_ => {},
	}
}