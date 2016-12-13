#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use bindings;

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

	// TODO: Add more errors - flush events not called (or should I just panic as that is a programmer error?)
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
pub type Result<T> = result::Result<T, Error>;

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

		let c_str: &CStr = unsafe { CStr::from_ptr(bindings::soundio_backend_name(self.clone().into())) };

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