extern crate libsoundio_sys as raw;

use std::ffi::CStr;
use std::fmt;

/// For your convenience, Native Endian and Foreign Endian constants are defined
/// which point to the respective SoundIoFormat values.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
	pub fn bytes_per_sample(&self) -> usize {
		unsafe { raw::soundio_get_bytes_per_sample((*self).into()) as usize }
	}
	pub fn bytes_per_frame(&self, channel_count: usize) -> usize {
		self.bytes_per_sample() * channel_count
	}
	pub fn bytes_per_second(&self, channel_count: usize, sample_rate: usize) -> usize {
		self.bytes_per_sample() * channel_count * sample_rate
	}
}

// Bit of a hack, but useful!
macro_rules! format_from_type {
    ($basic_type:ty, $format_type:path) => (
        impl From<$basic_type> for Format {
			fn from(_val: $basic_type) -> Format {
				$format_type
    		}
		}
	)
}

// I'm assuming little endian. If you're using big endian I pity you.
format_from_type!(i8, Format::S8);
format_from_type!(u8, Format::U8);
format_from_type!(i16, Format::S16LE);
format_from_type!(u16, Format::U16LE);
// TODO: What about i24?
format_from_type!(i32, Format::S32LE);
format_from_type!(u32, Format::U32LE);
format_from_type!(f32, Format::Float32LE);
format_from_type!(f64, Format::Float64LE);

pub trait CastF64 {
    fn from_f64(v: f64) -> Self;
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

impl fmt::Display for Format {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let c_str: &CStr = unsafe { CStr::from_ptr(raw::soundio_format_string((*self).into())) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		
		f.write_str(c_str.to_str().unwrap())
	}
}
