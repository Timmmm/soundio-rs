extern crate libsoundio_sys as raw;

use std::ffi::CStr;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Backend {
	None,
	Jack,
	PulseAudio,
	Alsa,
	CoreAudio,
	Wasapi,
	Dummy,
}

impl From<raw::SoundIoBackend> for Backend {
    fn from(backend: raw::SoundIoBackend) -> Backend {
		match backend {
			raw::SoundIoBackend::SoundIoBackendJack => Backend::Jack,
			raw::SoundIoBackend::SoundIoBackendPulseAudio => Backend::PulseAudio,
			raw::SoundIoBackend::SoundIoBackendAlsa => Backend::Alsa,
			raw::SoundIoBackend::SoundIoBackendCoreAudio => Backend::CoreAudio,
			raw::SoundIoBackend::SoundIoBackendWasapi => Backend::Wasapi,
			raw::SoundIoBackend::SoundIoBackendDummy => Backend::Dummy,
			_ => Backend::None,
		}
    }
}

impl From<Backend> for raw::SoundIoBackend {
    fn from(backend: Backend) -> raw::SoundIoBackend {
		match backend {
			Backend::Jack => raw::SoundIoBackend::SoundIoBackendJack,
			Backend::PulseAudio => raw::SoundIoBackend::SoundIoBackendPulseAudio,
			Backend::Alsa => raw::SoundIoBackend::SoundIoBackendAlsa,
			Backend::CoreAudio => raw::SoundIoBackend::SoundIoBackendCoreAudio,
			Backend::Wasapi => raw::SoundIoBackend::SoundIoBackendWasapi,
			Backend::Dummy => raw::SoundIoBackend::SoundIoBackendDummy,
			_ => raw::SoundIoBackend::SoundIoBackendNone,
		}
    }
}

impl fmt::Display for Backend {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// TODO: This may be overkill - I could just use the #[derive(Debug)] output; it's nearly identical.

		let c_str: &CStr = unsafe { CStr::from_ptr(raw::soundio_backend_name((*self).into())) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		
		f.write_str(c_str.to_str().unwrap())
	}
}