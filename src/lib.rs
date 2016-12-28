extern crate libsoundio_sys as raw;

mod types;
mod context;
mod device;
mod instream;
mod outstream;
mod util;
mod layout;
mod error;
mod channels;
mod backend;
mod format;

// TODO: Don't re-export absolutely everything.
pub use self::types::*;
pub use self::context::*;
pub use self::device::*;
pub use self::instream::*;
pub use self::outstream::*;
pub use self::util::*;
pub use self::layout::*;
pub use self::error::*;
pub use self::channels::*;
pub use self::backend::*;
pub use self::format::*;

/// Return the libsoundio version string, for example `"1.0.2"`.
///
/// # Examples
///
/// ```
/// println!("libsoundio version: {}", soundio::version_string());
/// ```
pub fn version_string() -> String {
	latin1_to_string( unsafe { raw::soundio_version_string() } )
}

/// Return the libsoundio version as a tuple, for exaample `(1, 0, 2)`.
///
/// # Examples
///
/// ```
/// let version = soundio::version();
/// if version.0 == 1 && version.1 == 1 {
/// 	println!("Congrats! You are using libsoundio 1.1.x");
/// }
/// ```
pub fn version() -> (i32, i32, i32) {
	unsafe {
		(
			raw::soundio_version_major() as i32,
			raw::soundio_version_minor() as i32,
			raw::soundio_version_patch() as i32,
		)
	}
}

/// Return `true` if libsoundio supports the given `Backend`.
///
/// TODO: How is this different to soundio::Context::available_backends()?
///
/// # Examples
///
/// ```
/// let backend_list = [
/// 	soundio::Backend::Jack,
/// 	soundio::Backend::PulseAudio,
/// 	soundio::Backend::Alsa,
/// 	soundio::Backend::CoreAudio,
/// 	soundio::Backend::Wasapi,
/// 	soundio::Backend::Dummy,
/// ];
///
/// for &backend in backend_list.iter() {
/// 	println!("Backend {} available? {}", backend, soundio::have_backend(backend));
/// } 
/// ```
pub fn have_backend(backend: Backend) -> bool {
	unsafe {
		raw::soundio_have_backend(backend.into()) != 0
	}
}


