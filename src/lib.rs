// Declare that there is a module at ./bindings/mod.rs
mod bindings; // TODO: Rename to 'c_ffi' or 'low level' or something.
// Declare that there is a module at ./soundio/mod.rs
mod soundio; // TODO: Rename to 'high level' or something.

// Import all the soundio::soundio::* symbols into soundio::, and make them public to users of this crate.
pub use soundio::*;