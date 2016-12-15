#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use bindings;

use super::types::*;
use super::device::*;

use std;
use std::mem;
use std::ffi::CStr;
use std::ptr;
use std::fmt;
use std::error;
use std::result;
use std::os::raw::{c_int, c_char, c_void, c_double};
use std::marker::PhantomData;

/// OutStream Callbacks
///
///
///
///

pub extern fn outstream_write_callback(stream: *mut bindings::SoundIoOutStream, frame_count_min: c_int, frame_count_max: c_int) {
	// Use stream.userdata to get a reference to the OutStream object.
	let raw_userdata_pointer = unsafe { (*stream).userdata as *mut OutStreamUserData };
	let userdata = unsafe { &mut (*raw_userdata_pointer) };

	let mut stream_writer = StreamWriter {
		outstream: userdata.outstream,
		frame_count_min: frame_count_min as _,
		frame_count_max: frame_count_max as _,
	};

	(userdata.write_callback)(&mut stream_writer);
}

pub extern fn outstream_underflow_callback(stream: *mut bindings::SoundIoOutStream) {
//	unimplemented!();
	println!("Outstream Underflow");
}

pub extern fn outstream_error_callback(stream: *mut bindings::SoundIoOutStream, err: c_int) {
//	unimplemented!();
	println!("Outstream Error: {}", Error::from(err))
}





/// OutStream
///
///
///
///


pub struct OutStream<'a> {
	pub userdata: Box<OutStreamUserData>,
	
	// This is just here to say that OutStream cannot outlive the Device it was created from.
	pub phantom: PhantomData<&'a Device<'a>>,
}

pub struct OutStreamUserData {
	pub outstream: *mut bindings::SoundIoOutStream,

	// TODO: Do these need to be thread-safe as write_callback() is called in a different thread?
	pub write_callback: Box<FnMut(&mut StreamWriter)>, // TODO: This shouldn't be an option.
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






/// StreamWriter
///
///
///
///

pub struct StreamWriter {
	// TODO: Add PhantomData...

	outstream: *mut bindings::SoundIoOutStream,
	frame_count_min: usize,
	frame_count_max: usize,

}

impl StreamWriter {
	// TODO: Somehow restrict this so you can't call it twice? Or just check that dynamically.
	// frame_count is the number of frames you want to write. It must be between
	// frame_count_min and frame_count_max.
	pub fn begin_write(&self, frame_count: usize) -> Result<ChannelAreas> {
		let mut areas: *mut bindings::SoundIoChannelArea = ptr::null_mut();
		let mut actual_frame_count: c_int = frame_count as _;

		match unsafe { bindings::soundio_outstream_begin_write(self.outstream, &mut areas as *mut _, &mut actual_frame_count as *mut _) } {
			0 => Ok( ChannelAreas {
				outstream: self.outstream,
				frame_count: actual_frame_count,
				areas: unsafe { Vec::from_raw_parts(areas, self.channel_count(), self.channel_count()) },
			} ),
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
			(*self.outstream).layout.channel_count as usize
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





/// ChannelAreas
///
///
///
///

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
		assert_eq!(self.areas[channel as usize].step as usize, std::mem::size_of::<T>());

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



