#![allow(dead_code)]

use bindings;

use super::types::*;
use super::util::*;
use super::outstream::*;
use super::context::*;

use std::ptr;
use std::os::raw::c_int;
use std::marker::PhantomData;


impl ChannelLayout {
	pub fn get_builtin() -> Vec<ChannelLayout> {

		unimplemented!();
	}

	pub fn get_default() -> ChannelLayout {

		unimplemented!();
	}

	pub fn best_matching_channel_layout(preferred_layouts: &Vec<ChannelLayout>, available_layouts: &Vec<ChannelLayout>) -> Option<ChannelLayout> {

		unimplemented!();
	}

	// This seems a bit unnecessary.
	pub fn find_channel(&self, channel: ChannelId) -> Option<usize> {

		unimplemented!();
	}

	// Populate the name field with the built-in name if this layout matches. Returns true if it did.
	pub fn detect_builtin(&mut self) -> bool {

		unimplemented!();
	}

	pub fn sort(layouts: &mut [ChannelLayout]) {

		unimplemented!();
	}
}