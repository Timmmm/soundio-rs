#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod bindings;

use std::mem;
use std::ffi::CStr;
use std::ptr;
use std::fmt;
use std::error;
use std::result;

use std::os::raw::{c_int, c_char};

#[derive(Debug, Copy, Clone)]
pub enum Error {
	/// Out of memory.
	NoMem,
	/// The backend does not appear to be active or running.
	InitAudioBackend,
	/// A system resource other than memory was not available.
	SystemResources,
	/// Attempted to open a device and failed.
	OpeningDevice,
	/// No device found.
	NoSuchDevice,
	/// The programmer did not comply with the API.
	Invalid,
	/// libsoundio was compiled without support for that backend.
	BackendUnavailable,
	/// An open stream had an error that can only be recovered from by
	/// destroying the stream and creating it again.
	Streaming,
	/// Attempted to use a device with parameters it cannot support.
	IncompatibleDevice,
	/// When JACK returns `JackNoSuchClient`
	NoSuchClient,
	/// Attempted to use parameters that the backend cannot support.
	IncompatibleBackend,
	/// Backend server shutdown or became inactive.
	BackendDisconnected,
	Interrupted,
	/// Buffer underrun occurred.
	Underflow,
	/// Unable to convert to or from UTF-8 to the native string format.
	EncodingString,

	/// Unknown error that libsoundio should never return.
	Unknown, 
}

impl From<c_int> for Error {
    fn from(err: c_int) -> Error {
		match err {
			// TODO: There must be a better way than this.
			1 => Error::NoMem,
			2 => Error::InitAudioBackend,
			3 => Error::SystemResources,
			4 => Error::OpeningDevice,
			5 => Error::NoSuchDevice,
			6 => Error::Invalid,
			7 => Error::BackendUnavailable,
			8 => Error::Streaming,
			9 => Error::IncompatibleDevice,
			10 => Error::NoSuchClient,
			11 => Error::IncompatibleBackend,
			12 => Error::BackendDisconnected,
			13 => Error::Interrupted,
			14 => Error::Underflow,
			15 => Error::EncodingString,
			_ => Error::Unknown,
		}
    }
}

impl From<Error> for c_int {
	fn from(err: Error) -> c_int {
		match err {
			// TODO: There must be a better way than this.
			Error::NoMem => 1,
			Error::InitAudioBackend => 2,
			Error::SystemResources => 3,
			Error::OpeningDevice => 4,
			Error::NoSuchDevice => 5,
			Error::Invalid => 6,
			Error::BackendUnavailable => 7,
			Error::Streaming => 8,
			Error::IncompatibleDevice => 9,
			Error::NoSuchClient => 10,
			Error::IncompatibleBackend => 11,
			Error::BackendDisconnected => 12,
			Error::Interrupted => 13,
			Error::Underflow => 14,
			Error::EncodingString => 15,
			Error::Unknown => -1, // This should never happen really.
		}
	}
}

// Local typedef for results that we return.
type Result<T> = result::Result<T, Error>;

// Implement displaying the error. We just use the description.
// TODO: There must be a way to automatically #[derive()] this no?
impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		f.write_str(self.description())
	}
}

// Implement the description for errors using soundio_strerror(), and the cause which we never know.
impl error::Error for Error {
	fn description(&self) -> &str {
		// TODO: I'm sure there is a simpler way than .clone().into(). I thought that by doing #[derive(Copy)]
		// it would automatically clone it...
		let c_str: &CStr = unsafe { CStr::from_ptr(bindings::soundio_strerror((*self).into())) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		c_str.to_str().unwrap()
	}

	fn cause(&self) -> Option<&error::Error> {
		// We never have any more cause information unfortunately.
		None
	}
}

impl From<Error> for String {
    fn from(err: Error) -> String {
		use std::error::Error;
		err.description().to_string()
    }
}

/// Specifies where a channel is physically located.
#[derive(Debug, Copy, Clone)]
pub enum ChannelId {
	Invalid,

	FrontLeft, // First of the more commonly supported ids.
	FrontRight,
	FrontCenter,
	Lfe,
	BackLeft,
	BackRight,
	FrontLeftCenter,
	FrontRightCenter,
	BackCenter,
	SideLeft,
	SideRight,
	TopCenter,
	TopFrontLeft,
	TopFrontCenter,
	TopFrontRight,
	TopBackLeft,
	TopBackCenter,
	TopBackRight, // Last of the more commonly supported ids.

	BackLeftCenter, // First of the less commonly supported ids.
	BackRightCenter,
	FrontLeftWide,
	FrontRightWide,
	FrontLeftHigh,
	FrontCenterHigh,
	FrontRightHigh,
	TopFrontLeftCenter,
	TopFrontRightCenter,
	TopSideLeft,
	TopSideRight,
	LeftLfe,
	RightLfe,
	Lfe2,
	BottomCenter,
	BottomLeftCenter,
	BottomRightCenter,

	// Mid/side recording
	MsMid,
	MsSide,

	// first order ambisonic channels
	AmbisonicW,
	AmbisonicX,
	AmbisonicY,
	AmbisonicZ,

	// X-Y Recording
	XyX,
	XyY,

	HeadphonesLeft, // First of the "other" channel ids
	HeadphonesRight,
	ClickTrack,
	ForeignLanguage,
	HearingImpaired,
	Narration,
	Haptic,
	DialogCentricMix, // Last of the "other" channel ids

	Aux,
	Aux0,
	Aux1,
	Aux2,
	Aux3,
	Aux4,
	Aux5,
	Aux6,
	Aux7,
	Aux8,
	Aux9,
	Aux10,
	Aux11,
	Aux12,
	Aux13,
	Aux14,
	Aux15,
}

/// Built-in channel layouts for convenience.
#[derive(Debug, Copy, Clone)]
pub enum ChannelLayoutId {
	Mono,
	Stereo,
	C2Point1,
	C3Point0,
	C3Point0Back,
	C3Point1,
	C4Point0,
	Quad,
	QuadSide,
	C4Point1,
	C5Point0Back,
	C5Point0Side,
	C5Point1,
	C5Point1Back,
	C6Point0Side,
	C6Point0Front,
	Hexagonal,
	C6Point1,
	C6Point1Back,
	C6Point1Front,
	C7Point0,
	C7Point0Front,
	C7Point1,
	C7Point1Wide,
	C7Point1WideBack,
	Octagonal,
}

// TODO: I am currently relying on the order of this matching the C API. I shouldn't.
#[derive(Debug, Copy, Clone)]
pub enum Backend {
	None,
	Jack,
	PulseAudio,
	Alsa,
	CoreAudio,
	Wasapi,
	Dummy,
}


impl From<bindings::SoundIoBackend> for Backend {
    fn from(backend: bindings::SoundIoBackend) -> Backend {
		match backend {
			bindings::SoundIoBackend::SoundIoBackendJack => Backend::Jack,
			bindings::SoundIoBackend::SoundIoBackendPulseAudio => Backend::PulseAudio,
			bindings::SoundIoBackend::SoundIoBackendAlsa => Backend::Alsa,
			bindings::SoundIoBackend::SoundIoBackendCoreAudio => Backend::CoreAudio,
			bindings::SoundIoBackend::SoundIoBackendWasapi => Backend::Wasapi,
			bindings::SoundIoBackend::SoundIoBackendDummy => Backend::Dummy,
			_ => Backend::None,
		}
    }
}

impl From<Backend> for bindings::SoundIoBackend {
    fn from(backend: Backend) -> bindings::SoundIoBackend {
		match backend {
			Backend::Jack => bindings::SoundIoBackend::SoundIoBackendJack,
			Backend::PulseAudio => bindings::SoundIoBackend::SoundIoBackendPulseAudio,
			Backend::Alsa => bindings::SoundIoBackend::SoundIoBackendAlsa,
			Backend::CoreAudio => bindings::SoundIoBackend::SoundIoBackendCoreAudio,
			Backend::Wasapi => bindings::SoundIoBackend::SoundIoBackendWasapi,
			Backend::Dummy => bindings::SoundIoBackend::SoundIoBackendDummy,
			_ => bindings::SoundIoBackend::SoundIoBackendNone,
		}
    }
}

impl fmt::Display for Backend {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// TODO: This may be overkill - I could just use the #[derive(Debug)] output; it's nearly identical.

		let c_str: &CStr = unsafe { CStr::from_ptr(bindings::soundio_backend_name(mem::transmute(*self as u32))) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		
		use std::error::Error;
		f.write_str(c_str.to_str().unwrap())
	}
}

pub enum DeviceAim {
	Input,  // capture / recording
	Output, // playback
}

/// For your convenience, Native Endian and Foreign Endian constants are defined
/// which point to the respective SoundIoFormat values.
#[derive(Debug, Copy, Clone)]
pub enum Format {
	Invalid,
	S8,        // Signed 8 bit
	U8,        // Unsigned 8 bit
	S16LE,     // Signed 16 bit Little Endian
	S16BE,     // Signed 16 bit Big Endian
	U16LE,     // Unsigned 16 bit Little Endian
	U16BE,     // Unsigned 16 bit Little Endian
	S24LE,     // Signed 24 bit Little Endian using low three bytes in 32-bit word
	S24BE,     // Signed 24 bit Big Endian using low three bytes in 32-bit word
	U24LE,     // Unsigned 24 bit Little Endian using low three bytes in 32-bit word
	U24BE,     // Unsigned 24 bit Big Endian using low three bytes in 32-bit word
	S32LE,     // Signed 32 bit Little Endian
	S32BE,     // Signed 32 bit Big Endian
	U32LE,     // Unsigned 32 bit Little Endian
	U32BE,     // Unsigned 32 bit Big Endian
	Float32LE, // Float 32 bit Little Endian, Range -1.0 to 1.0
	Float32BE, // Float 32 bit Big Endian, Range -1.0 to 1.0
	Float64LE, // Float 64 bit Little Endian, Range -1.0 to 1.0
	Float64BE, // Float 64 bit Big Endian, Range -1.0 to 1.0
}

#[derive(Debug)]
pub struct ChannelLayout {
	name: String,
	channels: Vec<ChannelId>,
}

#[derive(Debug, Copy, Clone)]
pub struct SampleRateRange {
	min: i32,
	max: i32,
}

pub struct Context {
	// The soundio library instance.
	soundio: *mut bindings::SoundIo,
	app_name: String,
}

extern fn on_backend_disconnect(sio: *mut bindings::SoundIo, err: c_int) {
	println!("Backend disconnected: {}", Error::from(err));
}

extern fn on_devices_change(sio: *mut bindings::SoundIo, err: c_int) {
	println!("Backend disconnected: {}", Error::from(err));
}

impl Context {

	pub fn new() -> Context {
		let soundio = unsafe { bindings::soundio_create() };
		if soundio == ptr::null_mut() {
			// Note that we should really abort() here (that's what the rest of Rust
			// does on OOM), but there is no stable way to abort in Rust that I can see.
			panic!("soundio_create() failed (out of memory).");
		}

		let context = Context { 
			soundio: soundio,
			app_name: String::new(),
		};
//		(*context.soundio).userdata = &context;

		// Note that the default on_backend_disconnect() handler panics! We'll change that.
		unsafe {
			(*context.soundio).on_backend_disconnect = on_backend_disconnect as *mut extern fn(*mut bindings::SoundIo, i32);
//			(*context.soundio).on_devices_change = on_devices_change as 
		}
		context
	}

	pub fn set_app_name(&mut self, name: String) {
		self.app_name = name;
//		(self.soundio).app_name = ???;
	}

	pub fn app_name(&self) -> String {
		self.app_name.clone()
	}

	pub fn connect(&mut self) -> Result<()> {
		let ret = unsafe { bindings::soundio_connect(self.soundio) };
		match ret {
			0 => Ok(()),
			_ => Err(ret.into()),
		}
	}

	pub fn connect_backend(&mut self, backend: Backend) -> Result<()> {
		let ret = unsafe { bindings::soundio_connect_backend(self.soundio, backend.into()) };
		match ret {
			0 => Ok(()),
			_ => Err(ret.into()),
		}
	}

	pub fn disconnect(&mut self) {
		unsafe {
			bindings::soundio_disconnect(self.soundio);
		}
	}

	pub fn current_backend(&mut self) -> Backend {
		unsafe {
			(*self.soundio).current_backend.into()
		}
	}

	pub fn available_backends(&mut self) -> Vec<Backend> {
		// TODO: Use soundio_backend_count() and soundio_get_backend().
		let count = unsafe { bindings::soundio_backend_count(self.soundio) };
		let mut backends = Vec::with_capacity(count as usize);
		for i in 0..count {
			backends.push( unsafe { bindings::soundio_get_backend(self.soundio, i).into() } );
		}
		backends
	}

	// You have to call this before enumerating devices.
	pub fn flush_events(&mut self) {
		unsafe {
			bindings::soundio_flush_events(self.soundio);
		}
	}

	pub fn wait_events(&mut self) {
		unsafe {
			bindings::soundio_wait_events(self.soundio);
		}
	}

	pub fn wakeup(&mut self) {
		unsafe {
			bindings::soundio_wakeup(self.soundio);
		}
	}

	pub fn force_device_scan(&mut self) {
		unsafe {
			bindings::soundio_force_device_scan(self.soundio);
		}
	}


	// Devices

	// Get a device, or None() if the index is out of bounds or you never called flush_events()
	// (you have to call flush_events() before getting devices)
	pub fn get_input_device(&mut self, index: usize) -> result::Result<Device, ()> {
		let device = unsafe { bindings::soundio_get_input_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(());
		}

		Ok(Device {
			device: device
		})
	}

	pub fn get_output_device(&mut self, index: usize) -> result::Result<Device, ()> {
		let device = unsafe { bindings::soundio_get_output_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(());
		}

		Ok(Device {
			device: device
		})
	}

	// Returns Err(()) if you never called flush_events().
	pub fn input_device_count(&mut self) -> result::Result<usize, ()> {
		let count = unsafe { bindings::soundio_input_device_count(self.soundio) };
		match count {
			-1 => Err(()),
			_ => Ok(count as usize),
		}
	}

	pub fn output_device_count(&mut self) -> result::Result<usize, ()> {
		let count = unsafe { bindings::soundio_output_device_count(self.soundio) };
		match count {
			-1 => Err(()),
			_ => Ok(count as usize),
		}
	}
	
	// Returns None if you never called flush_events().
	pub fn default_input_device_index(&mut self) -> result::Result<usize, ()> {
		let index = unsafe { bindings::soundio_default_input_device_index(self.soundio) };
		match index {
			-1 => Err(()),
			_ => Ok(index as usize),
		}
	}

	pub fn default_output_device_index(&mut self) -> result::Result<usize, ()> {
		let index = unsafe { bindings::soundio_default_output_device_index(self.soundio) };
		match index {
			-1 => Err(()),
			_ => Ok(index as usize),
		}
	}

	// Get all the input devices. If you never called flush_events() it returns Err(()).
	pub fn input_devices(&mut self) -> result::Result<Vec<Device>, ()> {
		let count = self.input_device_count()?;
		let mut devices = Vec::new();
		for i in 0..count {
			devices.push(self.get_input_device(i)?);
		}
		Ok(devices)
	}

	// Get all the output devices. If you never called flush_events() it returns Err(()).
	pub fn output_devices(&mut self) -> result::Result<Vec<Device>, ()> {
		let count = self.output_device_count()?;
		let mut devices = Vec::new();
		for i in 0..count {
			devices.push(self.get_output_device(i)?);
		}
		Ok(devices)
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_destroy(self.soundio);
		}
	}
}



pub struct Device {
	device: *mut bindings::SoundIoDevice,
}

impl Device {

	pub fn name(&self) -> String {
		latin1_to_string(unsafe { (*self.device).name } )
	}

	pub fn is_raw(&self) -> bool {
		unsafe { (*self.device).is_raw != 0 }
	}

	pub fn sort_channel_layouts(&mut self) {
		unsafe {
			bindings::soundio_device_sort_channel_layouts(self.device);
		}
	}

	pub fn supports_format(&mut self, format: Format) -> bool {
		false
	}

	// pub fn supports_layout(&mut self, layout: Layout) -> bool {
	// 	false
	// }

	pub fn supports_sample_rate(&mut self, sample_rate: i32) -> bool {
		false
	}

	pub fn nearest_sample_rate(&mut self, sample_rate: i32) -> i32 {
		0
	}
/*
	pub fn create_outstream(&mut self) -> OutStream {

	}

	pub fn create_instream(&mut self) -> InStream {

	}*/
}

impl Drop for Device {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_device_unref(self.device);
		}
	}
}


pub struct OutStream {
	outstream: *mut bindings::SoundIoOutStream,
}

// Outstream; copy this for instream.
impl OutStream {
	pub fn open(&mut self) -> Result<()> {
		Ok(())
	}
	pub fn start(&mut self) -> Result<()> {
		Ok(())
	}
	// Can only be called from the write_callback thread context.
	pub fn begin_write(&mut self) -> Result<()> {
		Ok(())
	}
	// Can only be called from the write_callback thread context.
	// TODO: In order to enforce that maybe I create another object and pass that object to the callback. 
	pub fn end_write(&mut self) -> Result<()> {
		Ok(())
	}

	pub fn clear_buffer(&mut self) -> Result<()> {
		Ok(())
	}

	pub fn pause(&mut self, pause: bool) -> Result<()> {
		Ok(())
	}

	pub fn get_latency(&mut self) -> Result<f64> {
		Ok(1.0)
	}
}

impl Drop for OutStream {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_outstream_destroy(self.outstream);
		}
	}
}







pub fn version_string() -> String {
	latin1_to_string(unsafe { bindings::soundio_version_string() } )
}


pub fn version() -> (i32, i32, i32) {
	unsafe {
		(
			bindings::soundio_version_major() as i32,
			bindings::soundio_version_minor() as i32,
			bindings::soundio_version_patch() as i32,
		)
	}
}

pub fn have_backend(backend: Backend) -> bool {
	unsafe {
		bindings::soundio_have_backend(backend.into()) != 0
	}
}





// Convert a Latin1 C String to a String.
// If `s` is null, an empty string is returned.
fn latin1_to_string(s: *const c_char) -> String {
	if s == ptr::null() {
		return String::new();
	}
	let c_str: &CStr = unsafe { CStr::from_ptr(s) };
	// This converts Latin1 to a String, instead of assuming UTF-8 (which I probably could to be fair).
	c_str.to_bytes().iter().map(|&c| c as char).collect()
}

// Convert a UTF-8 C String to a String.
// If `s` is null or contains invalid UTF-8, an empty string is returned.
fn utf8_to_string(s: *const c_char) -> String {
	if s == ptr::null() {
		return String::new();
	}
	let c_str: &CStr = unsafe { CStr::from_ptr(s) };

	c_str.to_str().unwrap_or("").to_string()
}