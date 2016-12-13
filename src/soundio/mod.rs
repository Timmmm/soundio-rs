#![allow(dead_code)]

// Declare that there is a module called types at ./types.rs
mod types;
mod context;
mod device;
mod instream;
mod outstream;

mod util;

// Import all the symbols from the types module, and make them public to others using this module.
pub use self::types::*;
pub use self::context::*;
pub use self::device::*;
pub use self::instream::*;
pub use self::outstream::*;

pub use self::util::*;

// Import the symbols from the <root>/bindings/mod.rs module
use bindings;

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


