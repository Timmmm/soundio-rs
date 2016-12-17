#![allow(dead_code)]

use bindings;

use super::device::*;
use super::error::*;
use super::types::*;

use std::ptr;
use std::os::raw::{c_int, c_double};
use std::marker::PhantomData;
use std::slice;
use std::mem;

pub extern fn outstream_write_callback(stream: *mut bindings::SoundIoOutStream, frame_count_min: c_int, frame_count_max: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut OutStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	let mut stream_writer = OutStreamWriter {
		outstream: userdata.outstream,
		frame_count_min: frame_count_min as _,
		frame_count_max: frame_count_max as _,
		write_started: false,
		channel_areas: Vec::new(),
		frame_count: 0,
		phantom: PhantomData,
	};

	(userdata.write_callback)(&mut stream_writer);

	// TODO: Something like this?
	// if stream_writer.write_started {
	// 	unsafe {
	// 		match bindings::soundio_outstream_end_write(stream_writer.outstream) {
	// 			0 => {},
	// 			x => panic!("Error writing outstream: {}", Error::from(x)),
	// 		}
	// 	}
	// }
}

pub extern fn outstream_underflow_callback(stream: *mut bindings::SoundIoOutStream) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut OutStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	if let Some(ref mut cb) = userdata.underflow_callback {
		cb();
	}
}

pub extern fn outstream_error_callback(stream: *mut bindings::SoundIoOutStream, err: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut OutStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	if let Some(ref mut cb) = userdata.error_callback {
		cb(err.into());
	}
}





/// OutStream represents an output stream for playback.
///
/// It is obtained from `Device` using `Device::open_outstream()` and
/// can be started, paused, and stopped.

pub struct OutStream<'a> {
	pub userdata: Box<OutStreamUserData>,
	
	// This is just here to say that OutStream cannot outlive the Device it was created from.
	pub phantom: PhantomData<&'a Device<'a>>,
}

// The callbacks required for an outstream are stored in this object. We also store a pointer
// to the raw outstream so that it can be passed to OutStreamWriter in the write callback.
pub struct OutStreamUserData {
	pub outstream: *mut bindings::SoundIoOutStream,

	// TODO: Do these need to be thread-safe as write_callback() is called in a different thread?
	pub write_callback: Box<FnMut(&mut OutStreamWriter)>,
	pub underflow_callback: Option<Box<FnMut()>>,
	pub error_callback: Option<Box<FnMut(Error)>>,
}

impl Drop for OutStreamUserData {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_outstream_destroy(self.outstream);
		}
	}
}

// Outstream; copy this for instream.
impl<'a> OutStream<'a> {
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


/// OutStreamWriter is passed to the write callback and can be used to write to the stream.
///
/// You start by calling `begin_write()` which yields a ChannelAreas object that you can
/// actually alter. When the ChannelAreas is dropped, the write is committed.
pub struct OutStreamWriter<'a> {
	outstream: *mut bindings::SoundIoOutStream,
	frame_count_min: usize,
	frame_count_max: usize,

	write_started: bool,

	// The memory area to write to - one for each channel. Populated after begin_write()
	channel_areas: Vec<bindings::SoundIoChannelArea>,
	// The actual frame count. Populated after begin_write()
	frame_count: usize,

	// This cannot outlive the scope that it is spawned from (in the write callback).
	phantom: PhantomData<&'a ()>,
}

impl<'a> OutStreamWriter<'a> {
	/// Start a write. You can only call this once per callback otherwise it panics.
	///
	/// frame_count is the number of frames you want to write. It must be between
	/// frame_count_min and frame_count_max.
	pub fn begin_write(&mut self, frame_count: usize) -> Result<()> {
		assert!(!self.write_started, "begin_write() called twice!");
		assert!(frame_count >= self.frame_count_min && frame_count <= self.frame_count_max, "frame_count out of range");

		let mut areas: *mut bindings::SoundIoChannelArea = ptr::null_mut();
		let mut actual_frame_count: c_int = frame_count as _;

		match unsafe { bindings::soundio_outstream_begin_write(self.outstream, &mut areas as *mut _, &mut actual_frame_count as *mut _) } {
			0 => {
				self.write_started = true;
				self.frame_count = actual_frame_count as _;
				let cc = self.channel_count();
				self.channel_areas = vec![bindings::SoundIoChannelArea { ptr: ptr::null_mut(), step: 0 }; cc];
				unsafe { self.channel_areas.copy_from_slice(slice::from_raw_parts::<bindings::SoundIoChannelArea>(areas, cc)); }
				Ok(())
			},
			e => Err(e.into()),
		}
	}
	
	/// Get the minimum frame count that you can call begin_write() with.
	pub fn frame_count_min(&self) -> usize {
		self.frame_count_min
	}

	/// Get the maximum frame count that you can call begin_write() with.
	pub fn frame_count_max(&self) -> usize {
		self.frame_count_max
	}

	/// Get the actual frame count that you did call begin_write() with. Panics if you haven't yet.
	pub fn frame_count(&self) -> usize {
		assert!(self.write_started);
		self.frame_count
	}

	// Get latency due to software only, not including hardware.
	pub fn software_latency(&self) -> f64 {
		unsafe {
			(*self.outstream).software_latency as _
		}
	}

	pub fn channel_count(&self) -> usize {
		unsafe {
			(*self.outstream).layout.channel_count as _
		}
	}

	pub fn sample_rate(&self) -> i32 {
		unsafe {
			(*self.outstream).sample_rate as _
		}
	}

	// Can only be called from the write_callback context. This includes both hardware and software latency.
	pub fn get_latency(&mut self) -> Result<f64> {
		let mut x: c_double = 0.0;
		match unsafe { bindings::soundio_outstream_get_latency(self.outstream, &mut x as *mut c_double) } {
			0 => Ok(x),
			e => Err(e.into()),
		}
	}

	/// Set the value of a sample/channel. Panics if out of range.
	pub fn set_sample<T: Copy>(&mut self, channel: usize, frame: usize, sample: T) {
		assert!(self.write_started);

		// Check format is at least the right size. This is only done in debug builds for speed reasons.
		debug_assert_eq!(mem::size_of::<T>(), Format::from(unsafe { (*self.outstream).format }).bytes_per_sample());

		assert!(channel < self.channel_count(), "Channel out of range");
		assert!(frame < self.frame_count(), "Frame out of range");

		unsafe {
			let ptr = self.channel_areas[channel].ptr.offset((frame * self.channel_areas[channel].step as usize) as isize) as *mut T;
			*ptr = sample;
		}
	}

	/// Get the value of a sample/channel. Panics if out of range.
	pub fn sample<T: Copy>(&self, channel: usize, frame: usize) -> T {
		assert!(self.write_started);

		// Check format is at least the right size. This is only done in debug builds for speed reasons.
		debug_assert_eq!(mem::size_of::<T>(), Format::from(unsafe { (*self.outstream).format }).bytes_per_sample());

		assert!(channel < self.channel_count(), "Channel out of range");
		assert!(frame < self.frame_count(), "Frame out of range");

		unsafe {
			let ptr = self.channel_areas[channel].ptr.offset((frame * self.channel_areas[channel].step as usize) as isize) as *mut T;
			*ptr
		}
	}
}

impl<'a> Drop for OutStreamWriter<'a> {
	fn drop(&mut self) {
		if self.write_started {
			unsafe {
				match bindings::soundio_outstream_end_write(self.outstream) {
					0 => {},
					x => panic!("Error writing outstream: {}", Error::from(x)),
				}
			}
		}
	}
}



