#![allow(dead_code)]

use bindings;

use super::error::*;
use super::types::*;

use std::ptr;
use std::os::raw::{c_int, c_double};
use std::marker::PhantomData;
use std::slice;
use std::mem;

pub extern fn instream_read_callback(stream: *mut bindings::SoundIoInStream, frame_count_min: c_int, frame_count_max: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut InStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	let mut stream_reader = InStreamReader {
		instream: userdata.instream,
		frame_count_min: frame_count_min as _,
		frame_count_max: frame_count_max as _,
		read_started: false,
		channel_areas: Vec::new(),
		frame_count: 0,
		phantom: PhantomData,
	};

	(userdata.read_callback)(&mut stream_reader);
}

pub extern fn instream_overflow_callback(stream: *mut bindings::SoundIoInStream) {
	// Use stream.userdata to get a reference to the InStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut InStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	if let Some(ref mut cb) = userdata.overflow_callback {
		cb();
	}
}

pub extern fn instream_error_callback(stream: *mut bindings::SoundIoInStream, err: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut InStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	if let Some(ref mut cb) = userdata.error_callback {
		cb(err.into());
	}
}

/// InStream represents an input stream for recording.
///
/// It is obtained from `Device` using `Device::open_instream()` and
/// can be started, paused, and stopped.

pub struct InStream<'a> {
	pub userdata: Box<InStreamUserData<'a>>,
	
	// This is just here to say that InStream cannot outlive the Device it was created from.
	pub phantom: PhantomData<&'a ()>,
}

// The callbacks required for an instream are stored in this object. We also store a pointer
// to the raw Instream so that it can be passed to InStreamReader in the write callback.
pub struct InStreamUserData<'a> {
	pub instream: *mut bindings::SoundIoInStream,

	pub read_callback: Box<FnMut(&mut InStreamReader) + 'a>,
	pub overflow_callback: Option<Box<FnMut() + 'a>>,
	pub error_callback: Option<Box<FnMut(Error) + 'a>>,
}

impl<'a> Drop for InStreamUserData<'a> {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_instream_destroy(self.instream);
		}
	}
}

impl<'a> InStream<'a> {
	pub fn start(&mut self) -> Result<()> {
		match unsafe { bindings::soundio_instream_start(self.userdata.instream) } {
			0 => Ok(()),
			x => Err(x.into()),
		}
	}

	pub fn pause(&mut self, pause: bool) -> Result<()> {
		match unsafe { bindings::soundio_instream_pause(self.userdata.instream, pause as i8) } {
			0 => Ok(()),
			e => Err(e.into()),
		}
	}
}

/// InStreamReader is passed to the read callback and can be used to read from the stream.
///
/// You start by calling `begin_read()` and then you can read the samples.
pub struct InStreamReader<'a> {
	instream: *mut bindings::SoundIoInStream,
	frame_count_min: usize,
	frame_count_max: usize,

	read_started: bool,

	// The memory area to write to - one for each channel. Populated after begin_read()
	channel_areas: Vec<bindings::SoundIoChannelArea>,
	// The actual frame count. Populated after begin_read()
	frame_count: usize,

	// This cannot outlive the scope that it is spawned from (in the write callback).
	phantom: PhantomData<&'a ()>,
}

impl<'a> InStreamReader<'a> {
	/// Start a read. You can only call this once per callback otherwise it panics.
	///
	/// frame_count is the number of frames you want to read. It must be between
	/// frame_count_min and frame_count_max.
	pub fn begin_read(&mut self, frame_count: usize) -> Result<()> {
		assert!(!self.read_started, "begin_read() called twice!");
		assert!(frame_count >= self.frame_count_min && frame_count <= self.frame_count_max, "frame_count out of range");

		let mut areas: *mut bindings::SoundIoChannelArea = ptr::null_mut();
		let mut actual_frame_count: c_int = frame_count as _;

		match unsafe { bindings::soundio_instream_begin_read(self.instream, &mut areas as *mut _, &mut actual_frame_count as *mut _) } {
			0 => {
				self.read_started = true;
				self.frame_count = actual_frame_count as _;
				let cc = self.channel_count();
				self.channel_areas = vec![bindings::SoundIoChannelArea { ptr: ptr::null_mut(), step: 0 }; cc];
				unsafe { self.channel_areas.copy_from_slice(slice::from_raw_parts::<bindings::SoundIoChannelArea>(areas, cc)); }
				Ok(())
			},
			e => Err(e.into()),
		}
	}
	
	/// Get the minimum frame count that you can call begin_read() with.
	pub fn frame_count_min(&self) -> usize {
		self.frame_count_min
	}

	/// Get the maximum frame count that you can call begin_read() with.
	pub fn frame_count_max(&self) -> usize {
		self.frame_count_max
	}

	/// Get the actual frame count that you did call begin_read() with. Panics if you haven't yet.
	pub fn frame_count(&self) -> usize {
		assert!(self.read_started);
		self.frame_count
	}

	// Get latency due to software only, not including hardware.
	pub fn software_latency(&self) -> f64 {
		unsafe {
			(*self.instream).software_latency as _
		}
	}

	pub fn channel_count(&self) -> usize {
		unsafe {
			(*self.instream).layout.channel_count as _
		}
	}

	pub fn sample_rate(&self) -> i32 {
		unsafe {
			(*self.instream).sample_rate as _
		}
	}

	// Can only be called from the read_callback context. This includes both hardware and software latency.
	pub fn get_latency(&mut self) -> Result<f64> {
		let mut x: c_double = 0.0;
		match unsafe { bindings::soundio_instream_get_latency(self.instream, &mut x as *mut c_double) } {
			0 => Ok(x),
			e => Err(e.into()),
		}
	}

	/// Set the value of a sample/channel. Panics if out of range or the wrong sized type (in debug builds).
	pub fn set_sample_typed<T: Copy>(&mut self, channel: usize, frame: usize, sample: T) {
		assert!(self.read_started);

		// Check format is at least the right size. This is only done in debug builds for speed reasons.
		debug_assert_eq!(mem::size_of::<T>(), Format::from(unsafe { (*self.instream).format }).bytes_per_sample());

		// TODO: Maybe actually we should just automatically convert it to the right type if it isn't already.

		assert!(channel < self.channel_count(), "Channel out of range");
		assert!(frame < self.frame_count(), "Frame out of range");

		unsafe {
			let ptr = self.channel_areas[channel].ptr.offset((frame * self.channel_areas[channel].step as usize) as isize) as *mut T;
			*ptr = sample;
		}
	}

	/// Get the value of a sample/channel. Panics if out of range or the wrong sized type (in debug builds).
	pub fn sample_typed<T: Copy>(&self, channel: usize, frame: usize) -> T {
		assert!(self.read_started);

		// Check format is at least the right size. This is only done in debug builds for speed reasons.
		debug_assert_eq!(mem::size_of::<T>(), Format::from(unsafe { (*self.instream).format }).bytes_per_sample());

		assert!(channel < self.channel_count(), "Channel out of range");
		assert!(frame < self.frame_count(), "Frame out of range");

		unsafe {
			let ptr = self.channel_areas[channel].ptr.offset((frame * self.channel_areas[channel].step as usize) as isize) as *mut T;
			*ptr
		}
	}

	/// Set the value of a sample/channel and coerces it to the correct type. Panics if out of range.
	// pub fn set_sample<T: CastF64>(&mut self, channel: usize, frame: usize, sample: T) {
	// 	match unsafe { (*self.instream).format } {
	// 		bindings::SoundIoFormat::SoundIoFormatS8 => self.set_sample_typed::<i8>(channel, frame, sample.from_f64()),
	/*		bindings::SoundIoFormat::SoundIoFormatU8 => Format::U8,
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
			bindings::SoundIoFormat::SoundIoFormatFloat64BE => Format::Float64BE,*/
	// 		_ => panic!("Unknown format"),			
	// 	}
	// }

	/// Get the value of a sample/channel as an f64. Panics if out of range.
	// pub fn sample(&self, channel: usize, frame: usize) -> f64 {
	// 	match unsafe { (*self.instream).format } {
    //         bindings::SoundIoFormat::SoundIoFormatS8 => self.sample_typed::<i8>(channel, frame).to_f64(),
	/*		bindings::SoundIoFormat::SoundIoFormatU8 => Format::U8,
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
			bindings::SoundIoFormat::SoundIoFormatFloat64BE => Format::Float64BE,*/
	// 		_ => panic!("Unknown format"),			
	// 	}
	// }
	fn foo() {

	}
}

impl<'a> Drop for InStreamReader<'a> {
	fn drop(&mut self) {
		if self.read_started {
			unsafe {
				match bindings::soundio_instream_end_read(self.instream) {
					0 => {},
					x => panic!("Error reading instream: {}", Error::from(x)),
				}
			}
		}
	}
}



