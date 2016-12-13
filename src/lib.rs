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

use std::os::raw::{c_int, c_char, c_void, c_double};

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

impl From<bindings::SoundIoFormat> for Format {
    fn from(format: bindings::SoundIoFormat) -> Format {
		match format {
			bindings::SoundIoFormat::SoundIoFormatS8 => Format::S8,
			bindings::SoundIoFormat::SoundIoFormatU8 => Format::U8,
			bindings::SoundIoFormat::SoundIoFormatS16LE => Format::S16LE,
			bindings::SoundIoFormat::SoundIoFormatS16BE => Format::S16BE,
			bindings::SoundIoFormat::SoundIoFormatU16LE => Format::U16LE,
			bindings::SoundIoFormat::SoundIoFormatU16BE => Format::U16BE,
			bindings::SoundIoFormat::SoundIoFormatS24LE => Format::S24LE,
			bindings::SoundIoFormat::SoundIoFormatS24BE => Format::S24BE,
			bindings::SoundIoFormat::SoundIoFormatU24LE => Format::U24LE,
			bindings::SoundIoFormat::SoundIoFormatU24BE => Format::U24BE,
			bindings::SoundIoFormat::SoundIoFormatS32LE => Format::S32LE,
			bindings::SoundIoFormat::SoundIoFormatS32BE => Format::S32BE,
			bindings::SoundIoFormat::SoundIoFormatU32LE => Format::U32LE,
			bindings::SoundIoFormat::SoundIoFormatU32BE => Format::U32BE,
			bindings::SoundIoFormat::SoundIoFormatFloat32LE => Format::Float32LE,
			bindings::SoundIoFormat::SoundIoFormatFloat32BE => Format::Float32BE,
			bindings::SoundIoFormat::SoundIoFormatFloat64LE => Format::Float64LE,
			bindings::SoundIoFormat::SoundIoFormatFloat64BE => Format::Float64BE,
			_ => Format::Invalid,
		}
    }
}

impl From<Format> for bindings::SoundIoFormat {
    fn from(format: Format) -> bindings::SoundIoFormat {
		match format {
			Format::S8 => bindings::SoundIoFormat::SoundIoFormatS8,
			Format::U8 => bindings::SoundIoFormat::SoundIoFormatU8,
			Format::S16LE => bindings::SoundIoFormat::SoundIoFormatS16LE,
			Format::S16BE => bindings::SoundIoFormat::SoundIoFormatS16BE,
			Format::U16LE => bindings::SoundIoFormat::SoundIoFormatU16LE,
			Format::U16BE => bindings::SoundIoFormat::SoundIoFormatU16BE,
			Format::S24LE => bindings::SoundIoFormat::SoundIoFormatS24LE,
			Format::S24BE => bindings::SoundIoFormat::SoundIoFormatS24BE,
			Format::U24LE => bindings::SoundIoFormat::SoundIoFormatU24LE,
			Format::U24BE => bindings::SoundIoFormat::SoundIoFormatU24BE,
			Format::S32LE => bindings::SoundIoFormat::SoundIoFormatS32LE,
			Format::S32BE => bindings::SoundIoFormat::SoundIoFormatS32BE,
			Format::U32LE => bindings::SoundIoFormat::SoundIoFormatU32LE,
			Format::U32BE => bindings::SoundIoFormat::SoundIoFormatU32BE,
			Format::Float32LE => bindings::SoundIoFormat::SoundIoFormatFloat32LE,
			Format::Float32BE => bindings::SoundIoFormat::SoundIoFormatFloat32BE,
			Format::Float64LE => bindings::SoundIoFormat::SoundIoFormatFloat64LE,
			Format::Float64BE => bindings::SoundIoFormat::SoundIoFormatFloat64BE,
			_ => bindings::SoundIoFormat::SoundIoFormatInvalid,
		}
    }
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
// 		unsafe { (*self.soundio).app_name = self.app_name.as_bytes() as *mut c_char; } // ?
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

	// TODO: I should use Result, but then just add another error: FlushNotCalled.

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

	// Get all the default input device. If you never called flush_events() it returns Err(()).
	pub fn default_input_device(&mut self) -> result::Result<Device, ()> {
		let index = self.default_input_device_index()?;
		Ok(self.get_input_device(index)?)
	}
	
	// Get all the default output device. If you never called flush_events() it returns Err(()).
	pub fn default_output_device(&mut self) -> result::Result<Device, ()> {
		let index = self.default_output_device_index()?;
		Ok(self.get_output_device(index)?)
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

	pub fn sort_channel_layouts(&self) {
		unsafe {
			bindings::soundio_device_sort_channel_layouts(self.device);
		}
	}

	pub fn supports_format(&self, format: Format) -> bool {
		unsafe {
			bindings::soundio_device_supports_format(self.device, format.into()) != 0
		}
	}

	// pub fn supports_layout(&mut self, layout: Layout) -> bool {
	// 	false
	// }

	pub fn supports_sample_rate(&self, sample_rate: i32) -> bool {
		unsafe {
			bindings::soundio_device_supports_sample_rate(self.device, sample_rate as c_int) != 0
		}
	}

	pub fn nearest_sample_rate(&self, sample_rate: i32) -> i32 {
		unsafe {
			bindings::soundio_device_nearest_sample_rate(self.device, sample_rate as c_int) as i32
		}
	}

	// TODO: Double check this
	pub fn open_outstream<CB: 'static + FnMut(&mut StreamWriter)>(&self,
			// sample_rate: i32,
			// format: Format,
			// layout: Layout,
			write_callback: CB) -> Result<OutStream> {
		let mut outstream = unsafe { bindings::soundio_outstream_create(self.device) };
		if outstream == ptr::null_mut() {
			// Note that we should really abort() here (that's what the rest of Rust
			// does on OOM), but there is no stable way to abort in Rust that I can see.
			panic!("soundio_outstream_create() failed (out of memory).");
		}

		// outstream.sample_rate = sample_rate;
		// outstream.format = format;
		// outstream.layout = layout;
		unsafe {
			(*outstream).software_latency = 0.0; // ?
			(*outstream).write_callback = outstream_write_callback as *mut _;
			(*outstream).underflow_callback = outstream_underflow_callback as *mut _;
			(*outstream).error_callback = outstream_error_callback as *mut _;
		}

		let stream = OutStream {
			userdata: Box::new( OutStreamUserData {
				outstream: outstream,
				write_callback: Box::new(write_callback),
				underflow_callback: None,
				error_callback: None,
			} ),
		};

		match unsafe { bindings::soundio_outstream_open(stream.userdata.outstream) } {
			0 => {},
			x => return Err(x.into()),
		};
		
		Ok(stream)
	}
/*
	pub fn open_instream(&mut self) -> InStream {

	}*/
}

extern fn outstream_write_callback(stream: *mut bindings::SoundIoOutStream, frame_count_min: c_int, frame_count_max: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut OutStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	let mut stream_writer = StreamWriter {
		outstream: userdata.outstream,
	};



	(userdata.write_callback)(&mut stream_writer);
}

extern fn outstream_underflow_callback(stream: *mut bindings::SoundIoOutStream) {

}

extern fn outstream_error_callback(stream: *mut bindings::SoundIoOutStream, err: c_int) {

}

impl Drop for Device {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_device_unref(self.device);
		}
	}
}

pub struct OutStreamUserData {
	outstream: *mut bindings::SoundIoOutStream,

	// TODO: Do these need to be thread-safe as write_callback() is called in a different thread?
	write_callback: Box<FnMut(&mut StreamWriter)>, // TODO: This shouldn't be an option.
	underflow_callback: Option<Box<FnMut()>>,
	error_callback: Option<Box<FnMut(Error)>>,
}

impl Drop for OutStreamUserData {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_outstream_destroy(self.outstream);
		}
	}
}

pub struct OutStream {
	userdata: Box<OutStreamUserData>,
}

// Outstream; copy this for instream.
impl OutStream {
	pub fn start(&mut self) -> Result<()> {
		match unsafe { bindings::soundio_outstream_start(self.userdata.outstream) } {
			0 => Ok(()),
			x => Err(x.into()),
		}
	}

	pub fn clear_buffer(&mut self) -> Result<()> {
		match unsafe { bindings::soundio_outstream_clear_buffer(self.userdata.outstream) } {
			0 => Ok(()),
			e => Err(e.into()),
		}
	}

	pub fn pause(&mut self, pause: bool) -> Result<()> {
		match unsafe { bindings::soundio_outstream_pause(self.userdata.outstream, pause as i8) } {
			0 => Ok(()),
			e => Err(e.into()),
		}
	}

}



pub struct StreamWriter {
	outstream: *mut bindings::SoundIoOutStream,
}

impl StreamWriter {
	// Begin write consumes this object so you can only call it once.
	// frame_count is the number of frames you want to write. It must be between
	// frame_count_min and frame_count_max.
	pub fn begin_write(self, frame_count: i32) -> Result<ChannelAreas> {
		let mut areas: *mut bindings::SoundIoChannelArea = ptr::null_mut();
		let mut actual_frame_count: c_int = frame_count;

		match unsafe { bindings::soundio_outstream_begin_write(self.outstream, &mut areas as *mut _, &mut actual_frame_count as *mut _) } {
			0 => Ok( ChannelAreas {
				outstream: self.outstream,
				frame_count: actual_frame_count,
				areas: vec![],
			} ),
			e => Err(e.into()),
		}
	}

	pub fn frame_count_min() -> i32 {
		0
	}

	pub fn frame_count_max() -> i32 {
		0
	}

	// Get latency due to software only, not including hardware.
	pub fn software_latency() -> f64 {
		0.0
	}

	// Can only be called from the write_callback context. This includes both hardware and software latency.
	pub fn get_latency(&mut self) -> Result<f64> {
		let mut x: c_double = 0.0;
		match unsafe { bindings::soundio_outstream_get_latency(self.outstream, &mut x as *mut c_double) } {
			0 => Ok(x),
			e => Err(e.into()),
		}
	}
}

pub struct ChannelAreas {
	outstream: *mut bindings::SoundIoOutStream,
	frame_count: i32,

	// The memory area to write to - one for each channel.
	areas: Vec<bindings::SoundIoChannelArea>,
}

impl ChannelAreas {
	pub fn frame_count(&self) -> i32 {
		self.frame_count
	}

	// Get the slice which we can write to.
	// T is the slice type they want.
	// TODO: Panic if the format is wrong?
	// TODO: Also panic if the step is not equal to sizeof(T).
	// TODO: Otherwise maybe we have to use a slice of structs, where the structs are
	// packet and have padding to take them up to step?
	pub fn get_slice<T>(&mut self, channel: i32) -> &mut [T] {
		assert_eq!(self.areas[channel as usize].step, mem::size_of::<T>());

		unsafe {
			std::slice::from_raw_parts_mut(self.areas[channel as usize].ptr as *mut T, self.frame_count as usize)
		}
	}

	pub fn get_step(&mut self, channel: i32) -> i32 {
		self.areas[channel as usize].step as i32
	}
}

impl Drop for ChannelAreas {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_outstream_end_write(self.outstream);
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