#![allow(dead_code)]

use bindings;

use super::device::*;
use super::error::*;
use super::channel_areas::*;

use std::ptr;
use std::os::raw::{c_int, c_double};
use std::marker::PhantomData;
use std::slice;

pub extern fn outstream_write_callback(stream: *mut bindings::SoundIoOutStream, frame_count_min: c_int, frame_count_max: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut OutStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	let mut stream_writer = OutStreamWriter {
		outstream: userdata.outstream,
		frame_count_min: frame_count_min as _,
		frame_count_max: frame_count_max as _,
		write_started: false,
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

	// This cannot outlive the scope that it is spawned from (in the write callback).
	phantom: PhantomData<&'a ()>,
}

impl<'a> OutStreamWriter<'a> {
	/// Start a write. You can only call this once per callback.
	// frame_count is the number of frames you want to write. It must be between
	// frame_count_min and frame_count_max.
	pub fn begin_write(&mut self, frame_count: usize) -> Result<ChannelAreas> {
		assert!(!self.write_started, "begin_write() called twice!");
		assert!(frame_count >= self.frame_count_min && frame_count <= self.frame_count_max, "frame_count out of range");

		let mut areas: *mut bindings::SoundIoChannelArea = ptr::null_mut();
		let mut actual_frame_count: c_int = frame_count as _;

		match unsafe { bindings::soundio_outstream_begin_write(self.outstream, &mut areas as *mut _, &mut actual_frame_count as *mut _) } {
			0 => {
				self.write_started = true;
				Ok( ChannelAreas {
					outstream: self.outstream,
					frame_count: actual_frame_count,
					areas: unsafe { 
						let mut a = vec![bindings::SoundIoChannelArea { ptr: ptr::null_mut(), step: 0 }; self.channel_count()];
						a.copy_from_slice(slice::from_raw_parts::<bindings::SoundIoChannelArea>(areas, self.channel_count()));
						a
					},
	//				phantom: PhantomData,
				} )
			},
			e => Err(e.into()),
		}
	}

	pub fn frame_count_min(&self) -> usize {
		self.frame_count_min
	}

	pub fn frame_count_max(&self) -> usize {
		self.frame_count_max
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
}






