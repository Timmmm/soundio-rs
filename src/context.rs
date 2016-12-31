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

/// Optional callback. Called when the backend disconnects. For example,
/// when the JACK server shuts down. When this happens, listing devices
/// and opening streams will always fail with
/// SoundIoErrorBackendDisconnected. This callback is only called during a
/// call to ::soundio_flush_events or ::soundio_wait_events.
/// If you do not supply a callback, the default will crash your program
/// with an error message. This callback is also called when the thread
/// that retrieves device information runs into an unrecoverable condition
/// such as running out of memory.
///
/// Possible errors:
/// * #SoundIoErrorBackendDisconnected
/// * #SoundIoErrorNoMem
/// * #SoundIoErrorSystemResources
/// * #SoundIoErrorOpeningDevice - unexpected problem accessing device
///   information
extern fn on_backend_disconnect(_sio: *mut raw::SoundIo, err: c_int) {
	// TODO: Allow user-defined callback.
	println!("Backend disconnected: {}", Error::from(err));
}

/// Called when the list of devices change. Only called
/// during a call to ::soundio_flush_events or ::soundio_wait_events.
extern fn on_devices_change(_sio: *mut raw::SoundIo) {
	println!("Devices changed");
}

/// Optional callback. Called from an unknown thread that you should not use
/// to call any soundio functions. You may use this to signal a condition
/// variable to wake up. Called when ::soundio_wait_events would be woken up.
extern fn on_events_signal(_sio: *mut raw::SoundIo) {

}

/// Optional: Real time priority warning.
/// This callback is fired when making thread real-time priority failed. By
/// default, it will print to stderr only the first time it is called
/// a message instructing the user how to configure their system to allow
/// real-time priority threads. This must be set to a function not NULL.
/// To silence the warning, assign this to a function that does nothing.
extern fn emit_rtprio_warning() {

}

impl Context {

	/// Create a new libsoundio context.
	///
	/// This panics if libsoundio fails to create the context object. This only happens due to out-of-memory conditions
	/// and Rust also panics (aborts actually) under those conditions in the standard library so this behaviour seemed acceptable.
	///
	/// You can create multiple `Context` instances to connect to multiple backends.
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

	/// Connect to the default backend, trying them in the order returned by `available_backends()`.
	/// It will fail with `Error::Invalid` if this instance is already connected to a backend.
	///
	/// # Return Values
	///
	/// * `soundio::Error::Invalid` if you are already connected.
	/// * `soundio::Error::NoMem`
	/// * `soundio::Error::SystemResources`
	/// * `soundio::Error::NoSuchClient` when JACK returns `JackNoSuchClient`.
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

	/// Connect to the specified backend. It will fail with `Error::Invalid` if this instance
	/// is already connected to a backend.
	///
	/// # Return Values
	///
	/// * `soundio::Error::Invalid` if you are already connected or the backend was invalid.
	/// * `soundio::Error::NoMem`
	/// * `soundio::Error::BackendUnavailable` if the backend was not compiled in.
	/// * `soundio::Error::SystemResources`
	/// * `soundio::Error::NoSuchClient` when JACK returns `JackNoSuchClient`.
	/// * `soundio::Error::InitAudioBackend` if the requested backend is not active.
	/// * `soundio::Error::BackendDisconnected` if the backend disconnected while connecting. See also [bug 103](https://github.com/andrewrk/libsoundio/issues/103)
	///
	/// # Examples
	///
	/// ```
	/// let mut ctx = soundio::Context::new();
	/// match ctx.connect_backend(soundio::Backend::Dummy) {
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

	/// Disconnect from the current backend. Does nothing if no backend is connected.
	/// It is usually not necessary to call this manually; the backend will disconnect
	/// automatically when `Context` is dropped.
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

	/// Return the current `Backend`.
	///
	/// If this `Context` isn't connected to any backend it returns `Backend::None`.
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
	/// println!("Available backends: {:?}", ctx.available_backends());
	/// ```
	pub fn available_backends(&self) -> Vec<Backend> {
		let count = unsafe { raw::soundio_backend_count(self.soundio) };
		let mut backends = Vec::with_capacity(count as usize);
		for i in 0..count {
			backends.push( unsafe { raw::soundio_get_backend(self.soundio, i).into() } );
		}
		backends
	}

	/// Atomically update information for all connected devices. Note that calling
	/// this function merely flips a pointer; the actual work of collecting device
	/// information is done elsewhere. It is performant to call this function many
	/// times per second.
	///
	/// When you call this, the following callbacks might be called:
	/// * SoundIo::on_devices_change
	/// * SoundIo::on_backend_disconnect
	/// This is the only time those callbacks can be called.
	///
	/// This must be called from the same thread as the thread in which you call
	/// these functions:
	/// * ::soundio_input_device_count
	/// * ::soundio_output_device_count
	/// * ::soundio_get_input_device
	/// * ::soundio_get_output_device
	/// * ::soundio_default_input_device_index
	/// * ::soundio_default_output_device_index
	///
	/// Note that if you do not care about learning about updated devices, you
	/// might call this function only once ever and never call
	/// ::soundio_wait_events.
	pub fn flush_events(&self) {
		unsafe {
			raw::soundio_flush_events(self.soundio);
		}
	}

	/// This function calls `flush_events` then blocks until another event
	/// is ready or you call `wakeup`. Be ready for spurious wakeups.
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

	/// If necessary you can manually trigger a device rescan. Normally you will
	/// not ever have to call this function, as libsoundio listens to system events
	/// for device changes and responds to them by rescanning devices and preparing
	/// the new device information for you to be atomically replaced when you call
	/// ::soundio_flush_events. However you might run into cases where you want to
	/// force trigger a device rescan, for example if an ALSA device has a
	/// SoundIoDevice::probe_error.
	///
	/// After you call this you still have to use ::soundio_flush_events or
	/// ::soundio_wait_events and then wait for the
	/// SoundIo::on_devices_change callback.
	///
	/// This can be called from any thread context except for
	/// SoundIoOutStream::write_callback and SoundIoInStream::read_callback
	pub fn force_device_scan(&self) {
		unsafe {
			raw::soundio_force_device_scan(self.soundio);
		}
	}

	// Get a device, or None if the index is out of bounds or you never called flush_events()
	// (you have to call flush_events() before getting devices).
	pub fn get_input_device(&self, index: usize) -> Result<Device> {
		let device = unsafe { raw::soundio_get_input_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(Error::OpeningDevice);
		}

		let probe_error = unsafe { (*device).probe_error };

		if probe_error != 0 {
			return Err(probe_error.into());
		}

		Ok(Device {
			device: device,
			phantom: PhantomData,
		})
	}

	pub fn get_output_device(&self, index: usize) -> Result<Device> {
		let device = unsafe { raw::soundio_get_output_device(self.soundio, index as c_int) };
		if device == ptr::null_mut() {
			return Err(Error::OpeningDevice);
		}

		let probe_error = unsafe { (*device).probe_error };

		if probe_error != 0 {
			return Err(probe_error.into());
		}

		Ok(Device {
			device: device,
			phantom: PhantomData,
		})
	}

	/// Panics if you never called flush_events().
	pub fn input_device_count(&self) -> usize {
		let count = unsafe { raw::soundio_input_device_count(self.soundio) };
		assert!(count != -1, "flush_events() must be called before input_device_count()");
		count as _
	}

	/// Panics if you never called flush_events().
	pub fn output_device_count(&self) -> usize {
		let count = unsafe { raw::soundio_output_device_count(self.soundio) };
		assert!(count != -1, "flush_events() must be called before output_device_count()");
		count as _
	}
	
	/// Returns None if there are no input devices, or you never called flush_events().
	pub fn default_input_device_index(&self) -> Option<usize> {
		let index = unsafe { raw::soundio_default_input_device_index(self.soundio) };
		match index {
			-1 => None,
			_ => Some(index as usize),
		}
	}

	/// Returns None if there are no input devices, or you never called flush_events().
	pub fn default_output_device_index(&self) -> Option<usize> {
		let index = unsafe { raw::soundio_default_output_device_index(self.soundio) };
		match index {
			-1 => None,
			_ => Some(index as usize),
		}
	}

	/// Get all the input devices. Panics if you never called flush_events().
	/// It returns an error if there is an error opening any of the devices.
	pub fn input_devices(&self) -> Result<Vec<Device>> {
		let count = self.input_device_count();
		let mut devices = Vec::new();
		for i in 0..count {
			devices.push(self.get_input_device(i)?);
		}
		Ok(devices)
	}

	/// Get all the input devices. Panics if you never called flush_events().
	/// It returns an error if there is an error opening any of the devices.
	pub fn output_devices(&self) -> Result<Vec<Device>> {
		let count = self.output_device_count();
		let mut devices = Vec::new();
		for i in 0..count {
			devices.push(self.get_output_device(i)?);
		}
		Ok(devices)
	}

	/// Get all the default input device. If you never called flush_events() it panics.
	/// If there are no devices it returns Error::NoSuchDevice.
	pub fn default_input_device(&self) -> Result<Device> {
		let index = match self.default_input_device_index() {
			Some(x) => x,
			None => return Err(Error::NoSuchDevice),
		};
		self.get_input_device(index)
	}
	
	/// Get all the default output device. If you never called flush_events() it panics.
	/// If there are no devices it returns Error::NoSuchDevice.
	pub fn default_output_device(&self) -> Result<Device> {
		let index = match self.default_output_device_index() {
			Some(x) => x,
			None => return Err(Error::NoSuchDevice),
		};
		self.get_output_device(index)
	}
}

impl Drop for Context {
	fn drop(&mut self) {
		unsafe {
			// This also disconnects if necessary.
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
	use super::super::backend::*;

    #[test]
    fn connect_default_backend() {
		let mut ctx = Context::new();
		match ctx.connect_backend(Backend::Dummy) {
			Ok(()) => println!("Connected to {}", ctx.current_backend()),
			Err(e) => println!("Couldn't connect: {}", e),
		}
    }

	#[test]
	fn available_backends() {
		let ctx = Context::new();
		println!("Available backends: {:?}", ctx.available_backends());
	}
}