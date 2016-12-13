#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use bindings;

use super::*;

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


