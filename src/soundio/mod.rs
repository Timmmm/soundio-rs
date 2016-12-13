#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// Declare that there is a module called types at ./types.rs
mod types;

// Import all the symbols from the types module, and make them public to others using this module.
pub use self::types::*;

// Import the symbols from the <root>/bindings/mod.rs module
use bindings;

use std;
use std::mem;
use std::ffi::CStr;
use std::ptr;
use std::fmt;
use std::error;
use std::result;
use std::os::raw::{c_int, c_char, c_void, c_double};

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