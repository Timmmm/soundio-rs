extern crate libsoundio_sys as raw;

use super::types::*;
use super::util::*;
use super::outstream::*;
use super::instream::*;
use super::error::*;
use super::layout::*;
use super::format::*;

use std::ptr;
use std::os::raw::c_int;
use std::marker::PhantomData;
use std::slice;

/// Device represents an input or output device.
///
/// It is obtained from a `Context` using `Context::get_input_device()` or `Context::get_output_device()`.
/// You can use it to open an input stream or output stream. 
pub struct Device<'a> {
	/// The raw pointer to the device.
	pub device: *mut raw::SoundIoDevice,

	/// This is just here to say that Device cannot outlive the Context it was created from.
	/// 'a is the lifetime of that Context.
	pub phantom: PhantomData<&'a ()>,
}

impl<'a> Device<'a> {

	/// A string of bytes that uniquely identifies this device.
	/// If the same physical device supports both input and output, that makes
	/// one SoundIoDevice for the input and one SoundIoDevice for the output.
	/// In this case, the id of each SoundIoDevice will be the same, and
	/// SoundIoDevice::aim will be different. Additionally, if the device
	/// supports raw mode, there may be up to four devices with the same id:
	/// one for each value of SoundIoDevice::is_raw and one for each value of
	/// SoundIoDevice::aim.
	pub fn id(&self) -> String {
		// This is not explicitly latin1 but it is described as 'a string of bytes' so
		// it may contain invalid UTF-8 sequences.
		latin1_to_string(unsafe { (*self.device).id } )
	}

	/// User-friendly UTF-8 encoded text to describe the device.
	pub fn name(&self) -> String {
		// This is explicitly UTF-8.
		utf8_to_string(unsafe { (*self.device).name } )
	}

	/// Tells whether this device is an input device or an output device.
	pub fn aim(&self) -> DeviceAim {
		unsafe {
			(*self.device).aim.into()
		}
	}

	/// Channel layouts are handled similarly to SoundIoDevice::formats.
	/// If this information is missing due to a SoundIoDevice::probe_error,
	/// layouts will be NULL. It's OK to modify this data, for example calling
	/// ::soundio_sort_channel_layouts on it.
	/// Devices are guaranteed to have at least 1 channel layout.
	pub fn layouts(&self) -> Vec<ChannelLayout> {

		let layouts_slice = unsafe {
			slice::from_raw_parts::<raw::SoundIoChannelLayout>((*self.device).layouts, (*self.device).layout_count as _)
		};

		layouts_slice.iter().map(|&x| x.into()).collect()
	}

	/// See SoundIoDevice::current_format apparently?
	pub fn current_layout(&self) -> ChannelLayout {
		unsafe { (*self.device).current_layout.into() }
	}

	/// List of formats this device supports. See also
	/// SoundIoDevice::current_format.
	pub fn formats(&self) -> Vec<Format> {

		let formats_slice = unsafe {
			slice::from_raw_parts::<raw::SoundIoFormat>((*self.device).formats, (*self.device).format_count as _)
		};

		formats_slice.iter().map(|&x| x.into()).collect()
	}

	/// A device is either a raw device or it is a virtual device that is
	/// provided by a software mixing service such as dmix or PulseAudio (see
	/// SoundIoDevice::is_raw). If it is a raw device,
	/// current_format is meaningless;
	/// the device has no current format until you open it. On the other hand,
	/// if it is a virtual device, current_format describes the
	/// destination sample format that your audio will be converted to. Or,
	/// if you're the lucky first application to open the device, you might
	/// cause the current_format to change to your format.
	/// Generally, you want to ignore current_format and use
	/// whatever format is most convenient
	/// for you which is supported by the device, because when you are the only
	/// application left, the mixer might decide to switch
	/// current_format to yours. You can learn the supported formats via
	/// formats and SoundIoDevice::format_count. If this information is missing
	/// due to a probe error, formats will be `NULL`. If current_format is
	/// unavailable, it will be set to #SoundIoFormatInvalid.
	/// Devices are guaranteed to have at least 1 format available.
	pub fn current_format(&self) -> Format {
		unsafe { (*self.device).current_format.into() }
	}

	/// Sample rate is the number of frames per second.
	/// Sample rate is handled very similar to SoundIoDevice::formats.
	/// If sample rate information is missing due to a probe error, the field
	/// will be set to NULL.
	/// Devices which have SoundIoDevice::probe_error set to #SoundIoErrorNone are
	/// guaranteed to have at least 1 sample rate available.
	pub fn sample_rates(&self) -> Vec<SampleRateRange> {

		let sample_rates_slice = unsafe {
			slice::from_raw_parts::<raw::SoundIoSampleRateRange>((*self.device).sample_rates, (*self.device).sample_rate_count as _)
		};

		sample_rates_slice.iter().map(|&x| x.into()).collect()
	}

	/// See SoundIoDevice::current_format
	/// 0 if sample rate information is missing due to a probe error.
	pub fn current_sample_rate(&self) -> i32 {
		unsafe { (*self.device).sample_rate_current as _ }
	}

    /// Software latency (current, minimum, maximum) in seconds. If this value is unknown or
    /// irrelevant, it is set to 0.0.
    /// For PulseAudio and WASAPI this value is unknown until you open a
    /// stream.
	pub fn software_latency(&self) -> SoftwareLatency {
		unsafe {
			SoftwareLatency {
				min: (*self.device).software_latency_min,
				max: (*self.device).software_latency_max,
				current: (*self.device).software_latency_current,
			}
		}
	}

	/// Raw means that you are directly opening the hardware device and not
	/// going through a proxy such as dmix, PulseAudio, or JACK. When you open a
	/// raw device, other applications on the computer are not able to
	/// simultaneously access the device. Raw devices do not perform automatic
	/// resampling and thus tend to have fewer formats available.
	pub fn is_raw(&self) -> bool {
		unsafe {
			(*self.device).is_raw != 0
		}
	}

	// TODO: probe_error?

	/// Sorts the channels returned by `layouts()` by channel count, descending.
	pub fn sort_channel_layouts(&self) {
		unsafe {
			raw::soundio_device_sort_channel_layouts(self.device);
		}
	}

	/// Returns whether or not a given sample `Format` is supported by this device.
	pub fn supports_format(&self, format: Format) -> bool {
		unsafe {
			raw::soundio_device_supports_format(self.device, format.into()) != 0
		}
	}

	/// Returns whether or not a given channel layout is supported by this device.
	pub fn supports_layout(&mut self, layout: ChannelLayout) -> bool {
		unsafe {
			// TODO: Check this cast is ok.
			raw::soundio_device_supports_layout(self.device, &layout.into() as *const _) != 0
		}
	}

	/// Returns true if the given sample rate is supported by this device.
	pub fn supports_sample_rate(&self, sample_rate: i32) -> bool {
		unsafe {
			raw::soundio_device_supports_sample_rate(self.device, sample_rate as c_int) != 0
		}
	}

	/// Returns the nearest supported sample rate of this device. Devices are guaranteed
	/// to support at least one sample rate.
	pub fn nearest_sample_rate(&self, sample_rate: i32) -> i32 {
		unsafe {
			raw::soundio_device_nearest_sample_rate(self.device, sample_rate as c_int) as i32
		}
	}


	/// After you call this function, SoundIoOutStream::software_latency is set to
	/// the correct value.
	///
	/// The next thing to do is call ::soundio_outstream_start.
	/// If this function returns an error, the outstream is in an invalid state and
	/// you must call ::soundio_outstream_destroy on it.
	///
	/// Possible errors:
	/// * #SoundIoErrorInvalid
	///   * SoundIoDevice::aim is not #SoundIoDeviceAimOutput
	///   * SoundIoOutStream::format is not valid
	///   * SoundIoOutStream::channel_count is greater than #SOUNDIO_MAX_CHANNELS
	/// * #SoundIoErrorNoMem
	/// * #SoundIoErrorOpeningDevice
	/// * #SoundIoErrorBackendDisconnected
	/// * #SoundIoErrorSystemResources
	/// * #SoundIoErrorNoSuchClient - when JACK returns `JackNoSuchClient`
	/// * #SoundIoErrorIncompatibleBackend - SoundIoOutStream::channel_count is
	///   greater than the number of channels the backend can handle.
	/// * #SoundIoErrorIncompatibleDevice - stream parameters requested are not
	///   compatible with the chosen device.

	// 'a is the lifetime of the Device. The OutStream lifetime 'b must be less than or equal to 'a (indicated by `'b: 'a`).
	// Also the callbacks must have a lifetime greate than or equal to 'b.
	// The callbacks only need to have the lifetime
	pub fn open_outstream<'b: 'a, WriteCB, UnderflowCB, ErrorCB>(
				&'a self,
				sample_rate: i32,
				format: Format,
				layout: ChannelLayout,
				latency: f64,
				write_callback: WriteCB,
				underflow_callback: Option<UnderflowCB>,
				error_callback: Option<ErrorCB>,
				) -> Result<OutStream<'b>>
		where
			WriteCB: 'b + FnMut(&mut OutStreamWriter),
			UnderflowCB: 'b + FnMut(),
			ErrorCB: 'b + FnMut(Error) {

		let mut outstream = unsafe { raw::soundio_outstream_create(self.device) };
		if outstream == ptr::null_mut() {
			// Note that we should really abort() here (that's what the rest of Rust
			// does on OOM), but there is no stable way to abort in Rust that I can see.
			panic!("soundio_outstream_create() failed (out of memory).");
		}

		unsafe {
			(*outstream).sample_rate = sample_rate; // TODO: Check this sample rate is supported.
			(*outstream).format = format.into(); // TODO: Check that this format is supported!
			(*outstream).layout = layout.into(); // TODO: Check that this layout is supported!
			(*outstream).software_latency = latency; // TODO: Should I set this?
			(*outstream).write_callback = outstream_write_callback as *mut _;
			(*outstream).underflow_callback = outstream_underflow_callback as *mut _;
			(*outstream).error_callback = outstream_error_callback as *mut _;

			// TODO: Allow setting (*outstream).name
			// TODO: Allow setting (*outstream).non_terminal_hint
		}

		let mut stream = OutStream {
			userdata: Box::new( OutStreamUserData {
				outstream: outstream,
				write_callback: Box::new(write_callback),
				underflow_callback: match underflow_callback {
					Some(cb) => Some(Box::new(cb)),
					None => None,
				},
				error_callback: match error_callback {
					Some(cb) => Some(Box::new(cb)),
					None => None,
				}
			} ),
			phantom: PhantomData,
		};

		// Safe userdata pointer.
		unsafe {
			(*stream.userdata.outstream).userdata = stream.userdata.as_mut() as *mut OutStreamUserData as *mut _;
		}

		match unsafe { raw::soundio_outstream_open(stream.userdata.outstream) } {
			0 => {},
			x => return Err(x.into()),
		};

		// TODO: Check this is the correct thing to do.
		match unsafe { (*stream.userdata.outstream).layout_error } {
			0 => {},
			x => return Err(x.into()),
		}
		
		Ok(stream)
	}


	// 'a is the lifetime of the Device. The InStream lifetime 'b must be less than or equal to 'a (indicated by `'b: 'a`).
	// Also the callbacks must have a lifetime greate than or equal to 'b.
	// The callbacks only need to have the lifetime
	pub fn open_instream<'b: 'a, ReadCB, OverflowCB, ErrorCB>(
				&'a self,
				sample_rate: i32,
				format: Format,
				layout: ChannelLayout,
				latency: f64,
				read_callback: ReadCB,
				overflow_callback: Option<OverflowCB>,
				error_callback: Option<ErrorCB>,
				) -> Result<InStream<'b>>
		where
			ReadCB: 'b + FnMut(&mut InStreamReader),
			OverflowCB: 'b + FnMut(),
			ErrorCB: 'b + FnMut(Error) {

		let mut instream = unsafe { raw::soundio_instream_create(self.device) };
		if instream == ptr::null_mut() {
			// Note that we should really abort() here (that's what the rest of Rust
			// does on OOM), but there is no stable way to abort in Rust that I can see.
			panic!("soundio_instream_create() failed (out of memory).");
		}

		unsafe {
			(*instream).sample_rate = sample_rate;
			(*instream).format = format.into();
			(*instream).layout = layout.into();
			(*instream).software_latency = latency;
			(*instream).read_callback = instream_read_callback as *mut _;
			(*instream).overflow_callback = instream_overflow_callback as *mut _;
			(*instream).error_callback = instream_error_callback as *mut _;
		}

		let mut stream = InStream {
			userdata: Box::new( InStreamUserData {
				instream: instream,
				read_callback: Box::new(read_callback),
				overflow_callback: match overflow_callback {
					Some(cb) => Some(Box::new(cb)),
					None => None,
				},
				error_callback: match error_callback {
					Some(cb) => Some(Box::new(cb)),
					None => None,
				}
			} ),
			phantom: PhantomData,
		};

		// Safe userdata pointer.
		unsafe {
			(*stream.userdata.instream).userdata = stream.userdata.as_mut() as *mut InStreamUserData as *mut _;
		}

		match unsafe { raw::soundio_instream_open(stream.userdata.instream) } {
			0 => {},
			x => return Err(x.into()),
		};

		// TODO: Check this is the correct thing to do.
		match unsafe { (*stream.userdata.instream).layout_error } {
			0 => {},
			x => return Err(x.into()),
		}
		
		Ok(stream)
	}
}

impl<'a> Drop for Device<'a> {
	fn drop(&mut self) {
		unsafe {
			raw::soundio_device_unref(self.device);
		}
	}
}
