extern crate soundio;

fn my_write_callback(stream: &mut soundio::StreamWriter) {
	println!("my_write_callback called!");
	let mut channel_areas = match stream.begin_write(stream.frame_count_max()) {
		Ok(x) => x,
		Err(e) => {
			println!("Error writing to stream: {}", e);
			return;
		}
	};

	let mut channel_left = channel_areas.get_slice(0);
	for i in 0..channel_left.len() {
		channel_left[i] = 0;
	}
}

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

	println!("Opening default output stream");
	let mut output_stream = output_dev.open_outstream(
		48000,
		soundio::Format::Float32LE,
		soundio::ChannelLayout::get_default(2),
		my_write_callback
	)?;

	println!("Starting stream");
	output_stream.start()?;

	for _ in 0..2 {
		ctx.wait_events();
	}

	Ok(())
}

fn main() {
	match run() {
		Err(x) => println!("Error: {}", x),
		_ => {},
	}
}