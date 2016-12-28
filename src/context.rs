extern crate libsoundio_sys as raw;

use super::device::*;
use super::error::*;
use super::backend::*;

use std::ptr;
use std::result;
use std::os::raw::{c_int, c_char};
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
	soundio: *mut raw::SoundIo,
	app_name: String,
}

extern fn on_backend_disconnect(_sio: *mut raw::SoundIo, err: c_int) {
	// TODO: Allow user-defined callback.
	println!("Backend disconnected: {}", Error::from(err));
}

extern fn on_devices_change(_sio: *mut raw::SoundIo) {
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
	pub fn new() -> Context {
		let soundio = unsafe { raw::soundio_create() };
		if soundio == ptr::null_mut() {
			// TODO: abort() here instead of panicking.
			panic!("soundio_create() failed (out of memory).");
		}

		let context = Context { 
			soundio: soundio,
			app_name: String::new(),
		};

		// Note that libsoundio's default on_backend_disconnect() handler panics!
		unsafe {
			(*context.soundio).on_backend_disconnect = on_backend_disconnect as *mut extern fn(*mut raw::SoundIo, i32);
			(*context.soundio).on_devices_change = on_devices_change as *mut extern fn(*mut raw::SoundIo);
			(*context.soundio).app_name = ptr::null_mut(); 

		// TODO: Save a reference here so that we can have user-defined callbacks (see OutStreamUserData).
		//   (*context.soundio).userdata = &context;
		}
		context
	}

	/// Set the app name. This is shown in JACK and some other backends. Any colons are removed. The max length is ? and the default is ?.
	/// It must be called before ?? 
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// ctx.set_app_name("My App");
	/// ```
	pub fn set_app_name(&mut self, name: &str) {
		self.app_name = name.chars().filter(|&x| x != ':').collect();
		unsafe { (*self.soundio).app_name = self.app_name.as_ptr() as *mut c_char; }
	}

	/// Get the app name previously set by `set_app_name()`.
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// assert_eq!(ctx.app_name(), "");
	/// ctx.set_app_name(":::My App:::");
	/// assert_eq!(ctx.app_name(), "My App");
	/// ```
	pub fn app_name(&self) -> String {
		self.app_name.clone()
	}

	/// Connect to the default backend. TODO: Which is the default backend? TODO: What if you're already connected?
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// match ctx.connect() {
	/// 	Ok(()) => println!("Connected to {}", ctx.current_backend()),
	/// 	Err(e) => println!("Couldn't connect: {}", e),
	/// }
	/// ```
	pub fn connect(&mut self) -> Result<()> {
		let ret = unsafe { raw::soundio_connect(self.soundio) };
		match ret {
			0 => Ok(()),
			_ => Err(ret.into()),
		}
	}

	/// Connect to the specified backend. TODO: What happens if you specify None? TODO: What if you're already connected?
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// match ctx.connect(soundio::Backend::Dummy) {
	/// 	Ok(()) => println!("Connected to dummy backend"),
	/// 	Err(e) => println!("Couldn't connect: {}", e),
	/// }
	/// ```
	pub fn connect_backend(&mut self, backend: Backend) -> Result<()> {
		let ret = unsafe { raw::soundio_connect_backend(self.soundio, backend.into()) };
		match ret {
			0 => Ok(()),
			_ => Err(ret.into()),
		}
	}

	/// Disconnect from the current backend. Does nothing (TODO: Check) if no backend is connected.
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// match ctx.connect() {
	/// 	Ok(()) => println!("Connected to {}", ctx.current_backend()),
	/// 	Err(e) => { println!("Couldn't connect: {}", e); return; },
	/// }
	/// ctx.disconnect();
	/// ```
	pub fn disconnect(&mut self) {
		unsafe {
			raw::soundio_disconnect(self.soundio);
		}
	}

	/// Return the current `Backend`, which may be `Backend::None`.
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// match ctx.connect() {
	/// 	Ok(()) => println!("Connected to {}", ctx.current_backend()),
	/// 	Err(e) => println!("Couldn't connect: {}", e),
	/// }
	/// ```
	pub fn current_backend(&self) -> Backend {
		unsafe {
			(*self.soundio).current_backend.into()
		}
	}

	/// Return a list of available backends on this system.
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// match ctx.connect() {
	/// 	Ok(()) => {
	/// 		println!("Connected to {}", ctx.current_backend());
	/// 		println!("Available backends: {}", ctx.available_backends());
	/// 	},
	/// 	Err(e) => println!("Couldn't connect: {}", e),
	/// }
	/// ```
	pub fn available_backends(&self) -> Vec<Backend> {
		let count = unsafe { raw::soundio_backend_count(self.soundio) };
		let mut backends = Vec::with_capacity(count as usize);
		for i in 0..count {
			backends.push( unsafe { raw::soundio_get_backend(self.soundio, i).into() } );
		}
		backends
	}

	/// Flush events. This must be called before enumerating devices.
	pub fn flush_events(&self) {
		unsafe {
			raw::soundio_flush_events(self.soundio);
		}
	}

	/// Wait for events. Call this in a loop.
	pub fn wait_events(&self) {
		unsafe {
			raw::soundio_wait_events(self.soundio);
		}
	}

	/// Wake up any other threads calling wait_events().
	/// TODO: For this to work, Context must be Send. I need to check exactly which functions can be called from
	/// different threads and then maybe separate Context into two objects, one that is Send/Sync and one that isn't.
	pub fn wakeup(&self) {
		unsafe {
			raw::soundio_wakeup(self.soundio);
		}
	}

	pub fn force_device_scan(&self) {
		unsafe {
			raw::soundio_force_device_scan(self.soundio);
		}
	}

	// Get a device, or None if the index is out of bounds or you never called flush_events()
	// (you have to call flush_events() before getting devices).
	pub fn get_input_device(&self, index: usize) -> result::Result<Device, ()> {
		let device = unsafe { raw::soundio_get_input_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(());
		}

		Ok(Device {
			device: device,
			phantom: PhantomData,
		})
	}

	pub fn get_output_device(&self, index: usize) -> result::Result<Device, ()> {
		let device = unsafe { raw::soundio_get_output_device(self.soundio, index as c_int) };
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
		let count = unsafe { raw::soundio_input_device_count(self.soundio) };
		match count {
			-1 => Err(()),
			_ => Ok(count as usize),
		}
	}

	pub fn output_device_count(&self) -> result::Result<usize, ()> {
		let count = unsafe { raw::soundio_output_device_count(self.soundio) };
		match count {
			-1 => Err(()),
			_ => Ok(count as usize),
		}
	}
	
	// Returns None if you never called flush_events().
	pub fn default_input_device_index(&self) -> result::Result<usize, ()> {
		let index = unsafe { raw::soundio_default_input_device_index(self.soundio) };
		match index {
			-1 => Err(()),
			_ => Ok(index as usize),
		}
	}

	pub fn default_output_device_index(&self) -> result::Result<usize, ()> {
		let index = unsafe { raw::soundio_default_output_device_index(self.soundio) };
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
			raw::soundio_destroy(self.soundio);
		}
	}
}

// This allows wakeup and wait_events to be called from other threads.
// TODO: Find out exactly the thread-safety properties of libsoundio.
unsafe impl Send for Context {}
unsafe impl Sync for Context {}


#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn connect_default_backend() {
		let mut ctx = Context::new();
		match ctx.connect() {
			Ok(()) => println!("Connected to {}", ctx.current_backend()),
			Err(e) => println!("Couldn't connect: {}", e),
		}
    }
}