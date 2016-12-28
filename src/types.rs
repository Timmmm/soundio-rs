extern crate libsoundio_sys as raw;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DeviceAim {
	Input,  // capture / recording
	Output, // playback
}

impl From<raw::SoundIoDeviceAim> for DeviceAim {
    fn from(aim: raw::SoundIoDeviceAim) -> DeviceAim {
		match aim {
			raw::SoundIoDeviceAim::SoundIoDeviceAimInput => DeviceAim::Input,
			raw::SoundIoDeviceAim::SoundIoDeviceAimOutput => DeviceAim::Output,
		}
    }
}

impl From<DeviceAim> for raw::SoundIoDeviceAim {
    fn from(aim: DeviceAim) -> raw::SoundIoDeviceAim {
		match aim {
			DeviceAim::Input => raw::SoundIoDeviceAim::SoundIoDeviceAimInput,
			DeviceAim::Output => raw::SoundIoDeviceAim::SoundIoDeviceAimOutput,
		}
    }
}

// TODO: Just use the standard range objects?
#[derive(Debug, Copy, Clone)]
pub struct SampleRateRange {
	pub min: i32,
	pub max: i32,
}