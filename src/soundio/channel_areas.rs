#![allow(dead_code)]

use bindings;

use super::error::*;

use std::mem;
use std::slice;
use std::marker::PhantomData;

/// ChannelAreas represents some raw memory that contains audio data. It can be read from
/// or written to and is used in the read and write callbacks.

pub struct ChannelAreas/*<'a>*/ {
	pub outstream: *mut bindings::SoundIoOutStream,
	pub frame_count: i32,

	// The memory area to write to - one for each channel.
	pub areas: Vec<bindings::SoundIoChannelArea>,

	// TODO: Make this work somehow.
//	pub phantom: PhantomData<&'a ()>,
}

impl/*<'a>*/ ChannelAreas/*<'a>*/ {
	/// Set the value of a sample/channel. Panics if out of range.
	pub fn set_sample<T: Copy>(&mut self, channel: usize, frame: usize, sample: T) {
		assert!(channel < self.channel_count(), "Channel out of range");
		assert!(frame < self.frame_count(), "Frame out of range");

		unsafe {
			let ptr = self.areas[channel].ptr.offset((frame * self.areas[channel].step as usize) as isize) as *mut T;
			*ptr = sample;
		}
	}

	/// Get the value of a sample/channel. Panics if out of range.
	pub fn sample<T: Copy>(&self, channel: usize, frame: usize) -> T {
		assert!(channel < self.channel_count(), "Channel out of range");
		assert!(frame < self.frame_count(), "Frame out of range");

		unsafe {
			let ptr = self.areas[channel].ptr.offset((frame * self.areas[channel].step as usize) as isize) as *mut T;
			*ptr
		}
	}
	
	/// Return the number of frames in this memory area.
	pub fn frame_count(&self) -> usize {
		self.frame_count as _
	}

	/// Return the number of channels in this area.
	pub fn channel_count(&self) -> usize {
		unsafe {
			(*self.outstream).layout.channel_count as _
		}
	}

	// TODO: Allow getting the channel IDs.

	// Get the slice which we can write to.
	// T is the slice type they want.
	// TODO: Panic if the format is wrong?
	// TODO: Also panic if the step is not equal to sizeof(T).
	// TODO: Otherwise maybe we have to use a slice of structs, where the structs are
	// packet and have padding to take them up to step?
	// pub fn get_slice<T>(&mut self, channel: i32) -> &mut [T] {
	// 	// TODO: This fails because the channels are interleaved. I'm not quite sure how to handle that.
	// 	assert_eq!(self.areas[channel as usize].step as usize, mem::size_of::<T>());

	// 	unsafe {
	// 		slice::from_raw_parts_mut(self.areas[channel as usize].ptr as *mut T, self.frame_count as usize)
	// 	}
	// }

	// pub fn get_step(&mut self, channel: i32) -> i32 {
	// 	self.areas[channel as usize].step as i32
	// }
}

// TODO: Is this really the best place for it? Maybe I should just have a flag in OutStreamWriter to say whether a write was started or not and call this manually in OutStreamWriter?
impl<'a> Drop for ChannelAreas/*<'a>*/ {
	fn drop(&mut self) {
		unsafe {
			match bindings::soundio_outstream_end_write(self.outstream) {
				0 => {},
				x => panic!("Error writing outstream: {}", Error::from(x)),
			}
		}
	}
}
