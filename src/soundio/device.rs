#![allow(dead_code)]

use bindings;

use super::types::*;
use super::util::*;
use super::outstream::*;
use super::error::*;
use super::layout::*;

use std::ptr;
use std::os::raw::c_int;
use std::marker::PhantomData;

pub struct Device<'a> {
	pub device: *mut bindings::SoundIoDevice,

	// This is just here to say that Device cannot outlive the Context it was created from. 'a is the lifetime of that Context.
	pub phantom: PhantomData<&'a ()>,
}

/// Device represents an input or output device.
///
/// It is obtained from a `Context` using `Context::get_input_device()` or `Context::get_output_device()`.
/// You can use it to open an input stream or output stream. 
impl<'a> Device<'a> {

	pub fn name(&self) -> String {
		latin1_to_string(unsafe { (*self.device).name } )
	}

	pub fn is_raw(&self) -> bool {
		unsafe {
			(*self.device).is_raw != 0
		}
	}

	pub fn aim(&self) -> DeviceAim {
		unsafe {
			(*self.device).aim.into()
		}
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

	pub fn supports_layout(&mut self, layout: ChannelLayout) -> bool {
		unsafe {
			// TODO: Check this cast is ok.
			bindings::soundio_device_supports_layout(self.device, &layout.into() as *const _) != 0
		}
	}

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

	// 'a is the lifetime of the Device. The OutStream lifetime 'b must be less than or equal to 'a (indicated by `'b: 'a`).
	// Also the callbacks must have a lifetime greate than or equal to 'b.
	// The callbacks only need to have the lifetime
	pub fn open_outstream<'b: 'a, WriteCB, UnderflowCB, ErrorCB>(
				&'a self,
				sample_rate: i32,
				format: Format,
				layout: ChannelLayout,
				latency: f64,
				write_callback: WriteCB,
				underflow_callback: Option<UnderflowCB>,
				error_callback: Option<ErrorCB>,
				) -> Result<OutStream<'b>>
		where
			WriteCB: 'b + FnMut(&mut OutStreamWriter),
			UnderflowCB: 'b + FnMut(),
			ErrorCB: 'b + FnMut(Error) {

		let mut outstream = unsafe { bindings::soundio_outstream_create(self.device) };
		if outstream == ptr::null_mut() {
			// Note that we should really abort() here (that's what the rest of Rust
			// does on OOM), but there is no stable way to abort in Rust that I can see.
			panic!("soundio_outstream_create() failed (out of memory).");
		}

		unsafe {
			(*outstream).sample_rate = sample_rate;
			(*outstream).format = format.into();
			(*outstream).layout = layout.into();
			(*outstream).software_latency = latency;
			(*outstream).write_callback = outstream_write_callback as *mut _;
			(*outstream).underflow_callback = outstream_underflow_callback as *mut _;
			(*outstream).error_callback = outstream_error_callback as *mut _;
		}

		let mut stream = OutStream {
			userdata: Box::new( OutStreamUserData {
				outstream: outstream,
				write_callback: Box::new(write_callback),
				underflow_callback: match underflow_callback {
					Some(cb) => Some(Box::new(cb)),
					None => None,
				},
				error_callback: match error_callback {
					Some(cb) => Some(Box::new(cb)),
					None => None,
				}
			} ),
			phantom: PhantomData,
		};

		// Safe userdata pointer.
		unsafe {
			(*stream.userdata.outstream).userdata = stream.userdata.as_mut() as *mut OutStreamUserData as *mut _;
		}

		match unsafe { bindings::soundio_outstream_open(stream.userdata.outstream) } {
			0 => {},
			x => return Err(x.into()),
		};

		// TODO: Check this is the correct thing to do.
		match unsafe { (*stream.userdata.outstream).layout_error } {
			0 => {},
			x => return Err(x.into()),
		}
		
		Ok(stream)
	}

	// TODO: Instream.
	// pub fn open_instream(&mut self) -> InStream {
	// 	unimplemented!();
	// }
}

impl<'a> Drop for Device<'a> {
	fn drop(&mut self) {
		unsafe {
			bindings::soundio_device_unref(self.device);
		}
	}
}
