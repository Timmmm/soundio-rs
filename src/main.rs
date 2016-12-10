extern crate soundio;


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

	let mut ctx = soundio::Context::new();

	println!("Available backends: {:?}", ctx.available_backends());

	ctx.connect()?;

	println!("Current backend: {:?}", ctx.current_backend());

	// We have to flush events so we can scan devices.
	println!("Flushing events.");
	ctx.flush_events();
	println!("Flushed");

	println!("Input device count: {}", ctx.input_device_count().unwrap_or(0));
	println!("Output device count: {}", ctx.output_device_count().unwrap_or(0));

	let output_devices = ctx.output_devices().map_err(|_| "Error getting output devices".to_string())?;

	for dev in output_devices {
		println!("Output device: {} {}", dev.name(), if dev.is_raw() { "raw" } else { "cooked" } );
	}

	Ok(())


}

fn main() {
	match run() {
		Err(x) => println!("Error: {}", x),
		_ => {},
	}
}