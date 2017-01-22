extern crate libsoundio_sys as raw;

use super::util::*;
use super::channels::*;

use std::os::raw::c_int;
use std::ptr;
use std::cmp::min;

/// A `ChannelLayout` specifies a number of channels, and the `ChannelId` of each channel.
/// A `ChannelLayout` also has a name, though it is really only for display purposes and does
/// not affect execution at any point.
///
/// For example, the built-in stereo layout that is returned by `ChannelLayout::get_builtin(`ChannelLayoutId::Stereo)` is equal to:
///
/// ```
/// ChannelLayout {
/// 	name: "Stereo".to_string(),
/// 	channels: vec![ChannelId::FrontLeft, ChannelId::FrontRight],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ChannelLayout {
	/// The name of the layout. This is mostly useful when enumerating built-in layouts.
	pub name: String,
	/// A list of channels. Order is significant.
	pub channels: Vec<ChannelId>,
}

impl From<raw::SoundIoChannelLayout> for ChannelLayout {
    fn from(layout: raw::SoundIoChannelLayout) -> ChannelLayout {
		ChannelLayout {
			name: latin1_to_string(layout.name),
			channels: layout.channels.iter().take(layout.channel_count as usize).map(|&x| x.into()).collect(),
		}
    }
}

impl From<ChannelLayout> for raw::SoundIoChannelLayout {
    fn from(layout: ChannelLayout) -> raw::SoundIoChannelLayout {
		raw::SoundIoChannelLayout {
			name: ptr::null(), // TODO: Allow the name to be set somehow? Do I need to?
			channel_count: layout.channels.len() as c_int,
			channels: {
				let mut c = [raw::SoundIoChannelId::SoundIoChannelIdInvalid; raw::SOUNDIO_MAX_CHANNELS];
				for i in 0..min(c.len(), layout.channels.len()) {
					c[i] = layout.channels[i].into();
				}
				c
			},
		}
    }
}

impl ChannelLayout {
	/// Get all of the built-in layouts.
	pub fn get_all_builtin() -> Vec<ChannelLayout> {
		let count = unsafe { raw::soundio_channel_layout_builtin_count() };
		let mut layouts = Vec::new();
		for i in 0..count {
			layouts.push( unsafe { (*raw::soundio_channel_layout_get_builtin(i)).into() } );
		}
		layouts
	}

	/// Get a specific built-in layout.
	pub fn get_builtin(id: ChannelLayoutId) -> ChannelLayout {
		unsafe {
			(*raw::soundio_channel_layout_get_builtin(
				raw::SoundIoChannelLayoutId::from(id) as _
				)).into()
		}
	}

	/// Get the default layout for the given number of channels.
	pub fn get_default(channel_count: i32) -> ChannelLayout {
		unsafe {
			(*raw::soundio_channel_layout_get_default(channel_count as c_int)).into()
		}
	}

	/// Iterates over preferred_layouts. Returns the first channel layout in
	/// preferred_layouts which matches (using ==) one of the channel layouts in
	/// available_layouts. Returns None if none matches.
	pub fn best_matching_channel_layout(preferred_layouts: &Vec<ChannelLayout>, available_layouts: &Vec<ChannelLayout>) -> Option<ChannelLayout> {
		for preferred_layout in preferred_layouts {
			if available_layouts.contains(preferred_layout) {
				return Some(preferred_layout.clone());
			}
		}
		None
	}

	/// Find the given channel in a layout and return its index, or `None` if it wasn't found.
	pub fn find_channel(&self, channel: ChannelId) -> Option<usize> {
		// There is a C function for this but it seems simpler and safer to do it in Rust.
		self.channels.iter().position(|&c| c == channel)
	}

	/// Populate the name field with the built-in name if this layout matches one of the built-in layouts.
	/// Returns `true` if it did.
	pub fn detect_builtin(&mut self) -> bool {
		let mut raw_layout = raw::SoundIoChannelLayout::from(self.clone());

		if unsafe { raw::soundio_channel_layout_detect_builtin(&mut raw_layout) } != 0 {
			self.name = latin1_to_string(raw_layout.name);
			return true;
		}
		false
	}

	/// Sort a set of `ChannelLayouts` by channel count, descending. The content of the channels
	/// and the layout name are ignored; only the number of channels is significant.
	pub fn sort(layouts: &mut [ChannelLayout]) {
		// This is easier to do in Rust. It literally sorts by channel count.
		layouts.sort_by(|a, b| a.channels.len().cmp(&b.channels.len()));
	}
}

/// Equality testing for layouts. The channels must be the same
/// IDs and in the same order. The layout name is ignored.
impl PartialEq for ChannelLayout {
    fn eq(&self, other: &ChannelLayout) -> bool {
        self.channels == other.channels
    }
}
impl Eq for ChannelLayout {}


/// Built-in channel layouts for convenience.
/// These can be used with `ChannelLayout::get_builtin()`.
///
/// Some values are prepended with `C` where they started with a digit. For example
/// `C2Point1` means 2.1 and so on.
///
/// # Examples
///
/// ```
/// println!("Stereo Layout: {:?}", soundio::ChannelLayout::get_builtin(soundio::ChannelLayoutId::Stereo));
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChannelLayoutId {
	Mono,
	Stereo,
	C2Point1, // Ignore the 'C'. It's just there because it can't start with a number.
	C3Point0,
	C3Point0Back,
	C3Point1,
	C4Point0,
	Quad,
	QuadSide,
	C4Point1,
	C5Point0Back,
	C5Point0Side,
	C5Point1,
	C5Point1Back,
	C6Point0Side,
	C6Point0Front,
	Hexagonal,
	C6Point1,
	C6Point1Back,
	C6Point1Front,
	C7Point0,
	C7Point0Front,
	C7Point1,
	C7Point1Wide,
	C7Point1WideBack,
	Octagonal,
}

impl From<raw::SoundIoChannelLayoutId> for ChannelLayoutId {
	fn from(channel_layout_id: raw::SoundIoChannelLayoutId) -> ChannelLayoutId {
		match channel_layout_id {
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdMono            => ChannelLayoutId::Mono,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdStereo          => ChannelLayoutId::Stereo,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId2Point1         => ChannelLayoutId::C2Point1,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0         => ChannelLayoutId::C3Point0,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0Back     => ChannelLayoutId::C3Point0Back,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point1         => ChannelLayoutId::C3Point1,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point0         => ChannelLayoutId::C4Point0,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuad            => ChannelLayoutId::Quad,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuadSide        => ChannelLayoutId::QuadSide,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point1         => ChannelLayoutId::C4Point1,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Back     => ChannelLayoutId::C5Point0Back,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Side     => ChannelLayoutId::C5Point0Side,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1         => ChannelLayoutId::C5Point1,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1Back     => ChannelLayoutId::C5Point1Back,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Side     => ChannelLayoutId::C6Point0Side,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Front    => ChannelLayoutId::C6Point0Front,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdHexagonal       => ChannelLayoutId::Hexagonal,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1         => ChannelLayoutId::C6Point1,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Back     => ChannelLayoutId::C6Point1Back,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Front    => ChannelLayoutId::C6Point1Front,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0         => ChannelLayoutId::C7Point0,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0Front    => ChannelLayoutId::C7Point0Front,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1         => ChannelLayoutId::C7Point1,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1Wide     => ChannelLayoutId::C7Point1Wide,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1WideBack => ChannelLayoutId::C7Point1WideBack,
			raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdOctagonal       => ChannelLayoutId::Octagonal,
		}
	}
}

impl From<ChannelLayoutId> for raw::SoundIoChannelLayoutId {
	fn from(channel_layout_id: ChannelLayoutId) -> raw::SoundIoChannelLayoutId {
		match channel_layout_id {
			ChannelLayoutId::Mono             => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdMono,
			ChannelLayoutId::Stereo           => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdStereo,
			ChannelLayoutId::C2Point1         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId2Point1,
			ChannelLayoutId::C3Point0         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0,
			ChannelLayoutId::C3Point0Back     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0Back,
			ChannelLayoutId::C3Point1         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point1,
			ChannelLayoutId::C4Point0         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point0,
			ChannelLayoutId::Quad             => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuad,
			ChannelLayoutId::QuadSide         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuadSide,
			ChannelLayoutId::C4Point1         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point1,
			ChannelLayoutId::C5Point0Back     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Back,
			ChannelLayoutId::C5Point0Side     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Side,
			ChannelLayoutId::C5Point1         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1,
			ChannelLayoutId::C5Point1Back     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1Back,
			ChannelLayoutId::C6Point0Side     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Side,
			ChannelLayoutId::C6Point0Front    => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Front,
			ChannelLayoutId::Hexagonal        => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdHexagonal,
			ChannelLayoutId::C6Point1         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1,
			ChannelLayoutId::C6Point1Back     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Back,
			ChannelLayoutId::C6Point1Front    => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Front,
			ChannelLayoutId::C7Point0         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0,
			ChannelLayoutId::C7Point0Front    => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0Front,
			ChannelLayoutId::C7Point1         => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1,
			ChannelLayoutId::C7Point1Wide     => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1Wide,
			ChannelLayoutId::C7Point1WideBack => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1WideBack,
			ChannelLayoutId::Octagonal        => raw::SoundIoChannelLayoutId::SoundIoChannelLayoutIdOctagonal,
		}
	}
}

