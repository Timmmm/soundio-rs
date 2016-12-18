#![allow(dead_code)]

use bindings;

use super::types::*;
use super::device::*;
use super::error::*;

use std::ptr;
use std::result;
use std::os::raw::c_int;
use std::marker::PhantomData;

/// `Context` represents the libsoundio library.
///
/// It must be created using `Context::new()` before most operations can be done and you
/// generally will only have one context object per app.
///
/// The underlying C struct is destroyed when this object is dropped, which means that it
/// must outlive all the Devices it creates. TODO: Enforce this using lifetimes.
///
/// # Examples
///
/// ```
/// let mut ctx = soundio::Context::new();
/// ```
pub struct Context {
	// The soundio library instance.
	soundio: *mut bindings::SoundIo,
	app_name: String,
}

extern fn on_backend_disconnect(_sio: *mut bindings::SoundIo, err: c_int) {
	// TODO: Allow user-defined callback.
	println!("Backend disconnected: {}", Error::from(err));
}

extern fn on_devices_change(_sio: *mut bindings::SoundIo) {
	println!("Devices changed");
}

impl Context {

	/// Create a new libsoundio context.
	///
	/// This panics if libsoundio fails to create the context object. This only happens due to out-of-memory conditions
	/// and Rust also panics (aborts actually) under those conditions in the standard library so this behaviour seemed acceptable.
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// ```
	pub fn new(app_name: &str) -> Context {
		let soundio = unsafe { bindings::soundio_create() };
		if soundio == ptr::null_mut() {
			// TODO: abort() here instead of panicking.
			panic!("soundio_create() failed (out of memory).");
		}

		let context = Context { 
			soundio: soundio,
			app_name: app_name.to_string(),
		};
		// TODO: Set app name in soundio.app_name.

		// TODO: Save a reference here so that we can have user-defined callbacks (see OutStreamUserData).
		//   (*context.soundio).userdata = &context;

		// Note that libsoundio's default on_backend_disconnect() handler panics!
		unsafe {
			(*context.soundio).on_backend_disconnect = on_backend_disconnect as *mut extern fn(*mut bindings::SoundIo, i32);
			(*context.soundio).on_devices_change = on_devices_change as *mut extern fn(*mut bindings::SoundIo);
		}
		context
	}

	/// Set the app name. This is shown in JACK and some other backends. Any colons are stripped. The max length is ? and the default is ?.
	pub fn set_app_name(&mut self, name: String) {
		self.app_name = name.chars().filter(|&x| x != ':').collect();
		// TODO: Actually set the app name in libsoundio. I need to understand lifetimes more for that...
		// Or maybe I just force the name to be 'static &str.
		//
		// Orrr maybe I box a str. Hmm yeah that probably makes the most sense I guess?
		//	unsafe { (*self.soundio).app_name = self.app_name.as_bytes() as *mut c_char; } // ?
	}

	/// Get the app name previously set by `set_app_name()`. 
	pub fn app_name(&self) -> String {
		self.app_name.clone()
	}

	/// Connect to the default backend.
	pub fn connect(&mut self) -> Result<()> {
		let ret = unsafe { bindings::soundio_connect(self.soundio) };
		match ret {
			0 => Ok(()),
			_ => Err(ret.into()),
		}
	}

	/// Connect to the specified backend.
	pub fn connect_backend(&mut self, backend: Backend) -> Result<()> {
		let ret = unsafe { bindings::soundio_connect_backend(self.soundio, backend.into()) };
		match ret {
			0 => Ok(()),
			_ => Err(ret.into()),
		}
	}

	/// Disconnect from the current backend. Does nothing (TODO: Check) if no backend is connected.
	pub fn disconnect(&mut self) {
		unsafe {
			bindings::soundio_disconnect(self.soundio);
		}
	}

	/// Return the current `Backend`, which may be `Backend::None`.
	pub fn current_backend(&self) -> Backend {
		unsafe {
			(*self.soundio).current_backend.into()
		}
	}

	/// Return a list of available backends on this system.
	pub fn available_backends(&self) -> Vec<Backend> {
		let count = unsafe { bindings::soundio_backend_count(self.soundio) };
		let mut backends = Vec::with_capacity(count as usize);
		for i in 0..count {
			backends.push( unsafe { bindings::soundio_get_backend(self.soundio, i).into() } );
		}
		backends
	}

	/// Flush events. This must be called before enumerating devices.
	pub fn flush_events(&self) {
		unsafe {
			bindings::soundio_flush_events(self.soundio);
		}
	}

	/// Wait for events. Call this in a loop.
	pub fn wait_events(&self) {
		unsafe {
			bindings::soundio_wait_events(self.soundio);
		}
	}

	/// Wake up any other threads calling wait_events().
	/// TODO: For this to work, Context must be Send. I need to check exactly which functions can be called from
	/// different threads and then maybe separate Context into two objects, one that is Send/Sync and one that isn't.
	pub fn wakeup(&self) {
		unsafe {
			bindings::soundio_wakeup(self.soundio);
		}
	}

	pub fn force_device_scan(&self) {
		unsafe {
			bindings::soundio_force_device_scan(self.soundio);
		}
	}


	// Get a device, or None if the index is out of bounds or you never called flush_events()
	// (you have to call flush_events() before getting devices).
	pub fn get_input_device(&self, index: usize) -> result::Result<Device, ()> {
		let device = unsafe { bindings::soundio_get_input_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(());
		}

		Ok(Device {
			device: device,
			phantom: PhantomData,
		})
	}

	pub fn get_output_device(&self, index: usize) -> result::Result<Device, ()> {
		let device = unsafe { bindings::soundio_get_output_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(());
		}

		Ok(Device {
			device: device,
			phantom: PhantomData,
		})
	}

	// TODO: I should use Result, but then just add another error: FlushNotCalled.
	// Or maybe just panic?

	// Returns Err(()) if you never called flush_events().
	pub fn input_device_count(&self) -> result::Result<usize, ()> {
		let count = unsafe { bindings::soundio_input_device_count(self.soundio) };
		match count {
			-1 => Err(()),
			_ => Ok(count as usize),
		}
	}

	pub fn output_device_count(&self) -> result::Result<usize, ()> {
		let count = unsafe { bindings::soundio_output_device_count(self.soundio) };
		match count {
			-1 => Err(()),
			_ => Ok(count as usize),
		}
	}
	
	// Returns None if you never called flush_events().
	pub fn default_input_device_index(&self) -> result::Result<usize, ()> {
		let index = unsafe { bindings::soundio_default_input_device_index(self.soundio) };
		match index {
			-1 => Err(()),
			_ => Ok(index as usize),
		}
	}

	pub fn default_output_device_index(&self) -> result::Result<usize, ()> {
		let index = unsafe { bindings::soundio_default_output_device_index(self.soundio) };
		match index {
			-1 => Err(()),
			_ => Ok(index as usize),
		}
	}

	// Get all the input devices. If you never called flush_events() it returns Err(()).
	pub fn input_devices(&self) -> result::Result<Vec<Device>, ()> {
		let count = self.input_device_count()?;
		let mut devices = Vec::new();
		for i in 0..count {
			devices.push(self.get_input_device(i)?);
		}
		Ok(devices)
	}

	// Get all the output devices. If you never called flush_events() it returns Err(()).
	pub fn output_devices(&self) -> result::Result<Vec<Device>, ()> {
		let count = self.output_device_count()?;
		let mut devices = Vec::new();
		for i in 0..count {
			devices.push(self.get_output_device(i)?);
		}
		Ok(devices)
	}

	// Get all the default input device. If you never called flush_events() it returns Err(()).
	pub fn default_input_device(&self) -> result::Result<Device, ()> {
		let index = self.default_input_device_index()?;
		Ok(self.get_input_device(index)?)
	}
	
	// Get all the default output device. If you never called flush_events() it returns Err(()).
	pub fn default_output_device(&self) -> result::Result<Device, ()> {
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

// This allows wakeup and wait_events to be called from other threads.
// TODO: Find out exactly the thread-safety properties of libsoundio.
unsafe impl Send for Context {}
unsafe impl Sync for Context {}