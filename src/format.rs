extern crate libsoundio_sys as raw;

use std::ffi::CStr;
use std::fmt;

/// Format defines the format of the samples. In 90% of cases you'll want `S16LE`, or maybe `Float64LE`.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Format {
	/// Invalid format
	Invalid,
	/// Signed 8 bit
	S8,
	/// Unsigned 8 bit
	U8,
	/// Signed 16 bit Little Endian
	S16LE,
	/// Signed 16 bit Big Endian
	S16BE,
	/// Unsigned 16 bit Little Endian
	U16LE,
	/// Unsigned 16 bit Big Endian
	U16BE,
	/// Signed 24 bit Little Endian using low three bytes in 32-bit word
	S24LE,
	/// Signed 24 bit Big Endian using low three bytes in 32-bit word
	S24BE,
	/// Unsigned 24 bit Little Endian using low three bytes in 32-bit word
	U24LE,
	/// Unsigned 24 bit Big Endian using low three bytes in 32-bit word
	U24BE,
	/// Signed 32 bit Little Endian
	S32LE,
	/// Signed 32 bit Big Endian
	S32BE,
	/// Unsigned 32 bit Little Endian
	U32LE,
	/// Unsigned 32 bit Big Endian
	U32BE,
	/// Float 32 bit Little Endian, Range -1.0 to 1.0
	Float32LE,
	/// Float 32 bit Big Endian, Range -1.0 to 1.0
	Float32BE,
	/// Float 64 bit Little Endian, Range -1.0 to 1.0
	Float64LE,
	/// Float 64 bit Big Endian, Range -1.0 to 1.0
	Float64BE,
}

// TODO: Add functions / constants for native / foreign endian. 

impl From<raw::SoundIoFormat> for Format {
	fn from(format: raw::SoundIoFormat) -> Format {
		match format {
			raw::SoundIoFormat::SoundIoFormatS8 => Format::S8,
			raw::SoundIoFormat::SoundIoFormatU8 => Format::U8,
			raw::SoundIoFormat::SoundIoFormatS16LE => Format::S16LE,
			raw::SoundIoFormat::SoundIoFormatS16BE => Format::S16BE,
			raw::SoundIoFormat::SoundIoFormatU16LE => Format::U16LE,
			raw::SoundIoFormat::SoundIoFormatU16BE => Format::U16BE,
			raw::SoundIoFormat::SoundIoFormatS24LE => Format::S24LE,
			raw::SoundIoFormat::SoundIoFormatS24BE => Format::S24BE,
			raw::SoundIoFormat::SoundIoFormatU24LE => Format::U24LE,
			raw::SoundIoFormat::SoundIoFormatU24BE => Format::U24BE,
			raw::SoundIoFormat::SoundIoFormatS32LE => Format::S32LE,
			raw::SoundIoFormat::SoundIoFormatS32BE => Format::S32BE,
			raw::SoundIoFormat::SoundIoFormatU32LE => Format::U32LE,
			raw::SoundIoFormat::SoundIoFormatU32BE => Format::U32BE,
			raw::SoundIoFormat::SoundIoFormatFloat32LE => Format::Float32LE,
			raw::SoundIoFormat::SoundIoFormatFloat32BE => Format::Float32BE,
			raw::SoundIoFormat::SoundIoFormatFloat64LE => Format::Float64LE,
			raw::SoundIoFormat::SoundIoFormatFloat64BE => Format::Float64BE,
			_ => Format::Invalid,
		}
	}
}

impl From<Format> for raw::SoundIoFormat {
	fn from(format: Format) -> raw::SoundIoFormat {
		match format {
			Format::S8 => raw::SoundIoFormat::SoundIoFormatS8,
			Format::U8 => raw::SoundIoFormat::SoundIoFormatU8,
			Format::S16LE => raw::SoundIoFormat::SoundIoFormatS16LE,
			Format::S16BE => raw::SoundIoFormat::SoundIoFormatS16BE,
			Format::U16LE => raw::SoundIoFormat::SoundIoFormatU16LE,
			Format::U16BE => raw::SoundIoFormat::SoundIoFormatU16BE,
			Format::S24LE => raw::SoundIoFormat::SoundIoFormatS24LE,
			Format::S24BE => raw::SoundIoFormat::SoundIoFormatS24BE,
			Format::U24LE => raw::SoundIoFormat::SoundIoFormatU24LE,
			Format::U24BE => raw::SoundIoFormat::SoundIoFormatU24BE,
			Format::S32LE => raw::SoundIoFormat::SoundIoFormatS32LE,
			Format::S32BE => raw::SoundIoFormat::SoundIoFormatS32BE,
			Format::U32LE => raw::SoundIoFormat::SoundIoFormatU32LE,
			Format::U32BE => raw::SoundIoFormat::SoundIoFormatU32BE,
			Format::Float32LE => raw::SoundIoFormat::SoundIoFormatFloat32LE,
			Format::Float32BE => raw::SoundIoFormat::SoundIoFormatFloat32BE,
			Format::Float64LE => raw::SoundIoFormat::SoundIoFormatFloat64LE,
			Format::Float64BE => raw::SoundIoFormat::SoundIoFormatFloat64BE,
			_ => raw::SoundIoFormat::SoundIoFormatInvalid,
		}
	}
}

impl Format {
	/// Returns the number of byte used per sample. Note that this
	/// is the size of the storage used for the sample, not the number of
	/// bits used. For example S24LE returns 4.
	///
	/// The returned values are specifically:
	///
	/// * S8: 1 
	/// * U8: 1 
	/// * S16LE: 2 
	/// * S16BE: 2 
	/// * U16LE: 2 
	/// * U16BE: 2 
	/// * S24LE: 4 
	/// * S24BE: 4 
	/// * U24LE: 4 
	/// * U24BE: 4 
	/// * S32LE: 4 
	/// * S32BE: 4 
	/// * U32LE: 4 
	/// * U32BE: 4 
	/// * Float32LE: 4 
	/// * Float32BE: 4 
	/// * Float64LE: 8 
	/// * Float64BE: 8 
	/// * Invalid: -1
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(Format::S8.bytes_per_sample(), 1);
	/// assert_eq!(Format::Float64LE.bytes_per_frame(), 8);
	/// ```
	pub fn bytes_per_sample(&self) -> usize {
		unsafe { raw::soundio_get_bytes_per_sample((*self).into()) as usize }
	}

	/// Returns the number of bytes per frame.
	/// A frame is one sample for all channels so this is simply the number
	/// of bytes for a sample `bytes_per_sample()` multiplied by the number of channels.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(Format::S8.bytes_per_frame(2), 2);
	/// assert_eq!(Format::Float64LE.bytes_per_frame(4), 32);
	/// ```
	pub fn bytes_per_frame(&self, channel_count: usize) -> usize {
		self.bytes_per_sample() * channel_count
	}

	/// Returns the number of bytes per second, which is the number of bytes
	/// per frame multiplied by the number of frames per second (the sample rate).
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(Format::S8.bytes_per_second(2, 8000), 16000);
	/// assert_eq!(Format::Float64LE.bytes_per_frame(4, 4000), 128000);
	/// ```
	pub fn bytes_per_second(&self, channel_count: usize, sample_rate: usize) -> usize {
		self.bytes_per_sample() * channel_count * sample_rate
	}
}

impl fmt::Display for Format {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let c_str: &CStr = unsafe { CStr::from_ptr(raw::soundio_format_string((*self).into())) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		
		f.write_str(c_str.to_str().unwrap())
	}
}

/// This is a trait for types that can be cast to or from an f64.
///
/// It is implemented for all the primitive numeric types: `u8`, `u16`, `u32`, `u64`,
/// `usize`, `i8`, `i16`, `i32`, `i64`, `isize`, `f32` and `f64`.
pub trait CastF64 {
	/// Cast the type from an f64. 
	fn from_f64(v: f64) -> Self;
	/// Cast the type to an f64.
	fn to_f64(v: Self) -> f64;
}

macro_rules! impl_cast_f64 {
	($($ty:ty)*) => {
		$(
			impl CastF64 for $ty {
				#[inline]
				fn from_f64(v: f64) -> $ty {
					v as $ty
				}

				#[inline]
				fn to_f64(v: $ty) -> f64 {
					v as f64
				}
			}
		)*
	}
}

impl_cast_f64!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);

