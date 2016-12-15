#![allow(dead_code)]

use bindings;

use super::types::*;

use std::os::raw::c_int;


impl ChannelLayout {
	pub fn get_builtin() -> Vec<ChannelLayout> {
		let count = unsafe { bindings::soundio_channel_layout_builtin_count() };
		let mut layouts = Vec::new();
		for i in 0..count {
			layouts.push( unsafe { (*bindings::soundio_channel_layout_get_builtin(i)).into() } );
		}
		layouts
	}

	pub fn get_default(channel_count: i32) -> ChannelLayout {
		unsafe {
			(*bindings::soundio_channel_layout_get_default(channel_count as c_int)).into()
		}
	}

	pub fn best_matching_channel_layout(preferred_layouts: &Vec<ChannelLayout>, available_layouts: &Vec<ChannelLayout>) -> Option<ChannelLayout> {

		unimplemented!();
	}

	// This seems a bit unnecessary.
	pub fn find_channel(&self, channel: ChannelId) -> Option<usize> {

		// There is a C function for this but it seems simpler to do it in Rust.
		self.channels.iter().position(|&c| c == channel)
	}

	// Populate the name field with the built-in name if this layout matches. Returns true if it did.
	pub fn detect_builtin(&mut self) -> bool {

		unimplemented!();
	}

	pub fn sort(layouts: &mut [ChannelLayout]) {

		unimplemented!();
	}
}