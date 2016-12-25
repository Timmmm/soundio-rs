#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate libsoundio_sys as raw;

use super::util::*;

use std::mem;
use std::ffi::CStr;
use std::ptr;
use std::fmt;
use std::error;
use std::result;
use std::cmp::min;

use std::os::raw::{c_int, c_char, c_void, c_double};

/// Specifies where a channel is physically located.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChannelId {
	Invalid,

	FrontLeft, // First of the more commonly supported ids.
	FrontRight,
	FrontCenter,
	Lfe,
	BackLeft,
	BackRight,
	FrontLeftCenter,
	FrontRightCenter,
	BackCenter,
	SideLeft,
	SideRight,
	TopCenter,
	TopFrontLeft,
	TopFrontCenter,
	TopFrontRight,
	TopBackLeft,
	TopBackCenter,
	TopBackRight, // Last of the more commonly supported ids.

	BackLeftCenter, // First of the less commonly supported ids.
	BackRightCenter,
	FrontLeftWide,
	FrontRightWide,
	FrontLeftHigh,
	FrontCenterHigh,
	FrontRightHigh,
	TopFrontLeftCenter,
	TopFrontRightCenter,
	TopSideLeft,
	TopSideRight,
	LeftLfe,
	RightLfe,
	Lfe2,
	BottomCenter,
	BottomLeftCenter,
	BottomRightCenter,

	// Mid/side recording
	MsMid,
	MsSide,

	// first order ambisonic channels
	AmbisonicW,
	AmbisonicX,
	AmbisonicY,
	AmbisonicZ,

	// X-Y Recording
	XyX,
	XyY,

	HeadphonesLeft, // First of the "other" channel ids
	HeadphonesRight,
	ClickTrack,
	ForeignLanguage,
	HearingImpaired,
	Narration,
	Haptic,
	DialogCentricMix, // Last of the "other" channel ids

	Aux,
	Aux0,
	Aux1,
	Aux2,
	Aux3,
	Aux4,
	Aux5,
	Aux6,
	Aux7,
	Aux8,
	Aux9,
	Aux10,
	Aux11,
	Aux12,
	Aux13,
	Aux14,
	Aux15,
}

impl From<raw::SoundIoChannelId> for ChannelId {
    fn from(channel_id: raw::SoundIoChannelId) -> ChannelId {
		match channel_id {
			raw::SoundIoChannelId::SoundIoChannelIdFrontLeft => ChannelId::FrontLeft,
			raw::SoundIoChannelId::SoundIoChannelIdFrontRight => ChannelId::FrontRight,
			raw::SoundIoChannelId::SoundIoChannelIdFrontCenter => ChannelId::FrontCenter,
			raw::SoundIoChannelId::SoundIoChannelIdLfe => ChannelId::Lfe,
			raw::SoundIoChannelId::SoundIoChannelIdBackLeft => ChannelId::BackLeft,
			raw::SoundIoChannelId::SoundIoChannelIdBackRight => ChannelId::BackRight,
			raw::SoundIoChannelId::SoundIoChannelIdFrontLeftCenter => ChannelId::FrontLeftCenter,
			raw::SoundIoChannelId::SoundIoChannelIdFrontRightCenter => ChannelId::FrontRightCenter,
			raw::SoundIoChannelId::SoundIoChannelIdBackCenter => ChannelId::BackCenter,
			raw::SoundIoChannelId::SoundIoChannelIdSideLeft => ChannelId::SideLeft,
			raw::SoundIoChannelId::SoundIoChannelIdSideRight => ChannelId::SideRight,
			raw::SoundIoChannelId::SoundIoChannelIdTopCenter => ChannelId::TopCenter,
			raw::SoundIoChannelId::SoundIoChannelIdTopFrontLeft => ChannelId::TopFrontLeft,
			raw::SoundIoChannelId::SoundIoChannelIdTopFrontCenter => ChannelId::TopFrontCenter,
			raw::SoundIoChannelId::SoundIoChannelIdTopFrontRight => ChannelId::TopFrontRight,
			raw::SoundIoChannelId::SoundIoChannelIdTopBackLeft => ChannelId::TopBackLeft,
			raw::SoundIoChannelId::SoundIoChannelIdTopBackCenter => ChannelId::TopBackCenter,
			raw::SoundIoChannelId::SoundIoChannelIdTopBackRight  => ChannelId::TopBackRight ,
			raw::SoundIoChannelId::SoundIoChannelIdBackLeftCenter => ChannelId::BackLeftCenter,
			raw::SoundIoChannelId::SoundIoChannelIdBackRightCenter => ChannelId::BackRightCenter,
			raw::SoundIoChannelId::SoundIoChannelIdFrontLeftWide => ChannelId::FrontLeftWide,
			raw::SoundIoChannelId::SoundIoChannelIdFrontRightWide => ChannelId::FrontRightWide,
			raw::SoundIoChannelId::SoundIoChannelIdFrontLeftHigh => ChannelId::FrontLeftHigh,
			raw::SoundIoChannelId::SoundIoChannelIdFrontCenterHigh => ChannelId::FrontCenterHigh,
			raw::SoundIoChannelId::SoundIoChannelIdFrontRightHigh => ChannelId::FrontRightHigh,
			raw::SoundIoChannelId::SoundIoChannelIdTopFrontLeftCenter => ChannelId::TopFrontLeftCenter,
			raw::SoundIoChannelId::SoundIoChannelIdTopFrontRightCenter => ChannelId::TopFrontRightCenter,
			raw::SoundIoChannelId::SoundIoChannelIdTopSideLeft => ChannelId::TopSideLeft,
			raw::SoundIoChannelId::SoundIoChannelIdTopSideRight => ChannelId::TopSideRight,
			raw::SoundIoChannelId::SoundIoChannelIdLeftLfe => ChannelId::LeftLfe,
			raw::SoundIoChannelId::SoundIoChannelIdRightLfe => ChannelId::RightLfe,
			raw::SoundIoChannelId::SoundIoChannelIdLfe2 => ChannelId::Lfe2,
			raw::SoundIoChannelId::SoundIoChannelIdBottomCenter => ChannelId::BottomCenter,
			raw::SoundIoChannelId::SoundIoChannelIdBottomLeftCenter => ChannelId::BottomLeftCenter,
			raw::SoundIoChannelId::SoundIoChannelIdBottomRightCenter => ChannelId::BottomRightCenter,
			raw::SoundIoChannelId::SoundIoChannelIdMsMid => ChannelId::MsMid,
			raw::SoundIoChannelId::SoundIoChannelIdMsSide => ChannelId::MsSide,
			raw::SoundIoChannelId::SoundIoChannelIdAmbisonicW => ChannelId::AmbisonicW,
			raw::SoundIoChannelId::SoundIoChannelIdAmbisonicX => ChannelId::AmbisonicX,
			raw::SoundIoChannelId::SoundIoChannelIdAmbisonicY => ChannelId::AmbisonicY,
			raw::SoundIoChannelId::SoundIoChannelIdAmbisonicZ => ChannelId::AmbisonicZ,
			raw::SoundIoChannelId::SoundIoChannelIdXyX => ChannelId::XyX,
			raw::SoundIoChannelId::SoundIoChannelIdXyY => ChannelId::XyY,
			raw::SoundIoChannelId::SoundIoChannelIdHeadphonesLeft => ChannelId::HeadphonesLeft,
			raw::SoundIoChannelId::SoundIoChannelIdHeadphonesRight => ChannelId::HeadphonesRight,
			raw::SoundIoChannelId::SoundIoChannelIdClickTrack => ChannelId::ClickTrack,
			raw::SoundIoChannelId::SoundIoChannelIdForeignLanguage => ChannelId::ForeignLanguage,
			raw::SoundIoChannelId::SoundIoChannelIdHearingImpaired => ChannelId::HearingImpaired,
			raw::SoundIoChannelId::SoundIoChannelIdNarration => ChannelId::Narration,
			raw::SoundIoChannelId::SoundIoChannelIdHaptic => ChannelId::Haptic,
			raw::SoundIoChannelId::SoundIoChannelIdDialogCentricMix  => ChannelId::DialogCentricMix ,
			raw::SoundIoChannelId::SoundIoChannelIdAux => ChannelId::Aux,
			raw::SoundIoChannelId::SoundIoChannelIdAux0 => ChannelId::Aux0,
			raw::SoundIoChannelId::SoundIoChannelIdAux1 => ChannelId::Aux1,
			raw::SoundIoChannelId::SoundIoChannelIdAux2 => ChannelId::Aux2,
			raw::SoundIoChannelId::SoundIoChannelIdAux3 => ChannelId::Aux3,
			raw::SoundIoChannelId::SoundIoChannelIdAux4 => ChannelId::Aux4,
			raw::SoundIoChannelId::SoundIoChannelIdAux5 => ChannelId::Aux5,
			raw::SoundIoChannelId::SoundIoChannelIdAux6 => ChannelId::Aux6,
			raw::SoundIoChannelId::SoundIoChannelIdAux7 => ChannelId::Aux7,
			raw::SoundIoChannelId::SoundIoChannelIdAux8 => ChannelId::Aux8,
			raw::SoundIoChannelId::SoundIoChannelIdAux9 => ChannelId::Aux9,
			raw::SoundIoChannelId::SoundIoChannelIdAux10 => ChannelId::Aux10,
			raw::SoundIoChannelId::SoundIoChannelIdAux11 => ChannelId::Aux11,
			raw::SoundIoChannelId::SoundIoChannelIdAux12 => ChannelId::Aux12,
			raw::SoundIoChannelId::SoundIoChannelIdAux13 => ChannelId::Aux13,
			raw::SoundIoChannelId::SoundIoChannelIdAux14 => ChannelId::Aux14,
			raw::SoundIoChannelId::SoundIoChannelIdAux15 => ChannelId::Aux15,
			_ => ChannelId::Invalid,
		}
    }
}

impl From<ChannelId> for raw::SoundIoChannelId {
    fn from(channel_id: ChannelId) -> raw::SoundIoChannelId {
		match channel_id {
			ChannelId::FrontLeft => raw::SoundIoChannelId::SoundIoChannelIdFrontLeft,
			ChannelId::FrontRight => raw::SoundIoChannelId::SoundIoChannelIdFrontRight,
			ChannelId::FrontCenter => raw::SoundIoChannelId::SoundIoChannelIdFrontCenter,
			ChannelId::Lfe => raw::SoundIoChannelId::SoundIoChannelIdLfe,
			ChannelId::BackLeft => raw::SoundIoChannelId::SoundIoChannelIdBackLeft,
			ChannelId::BackRight => raw::SoundIoChannelId::SoundIoChannelIdBackRight,
			ChannelId::FrontLeftCenter => raw::SoundIoChannelId::SoundIoChannelIdFrontLeftCenter,
			ChannelId::FrontRightCenter => raw::SoundIoChannelId::SoundIoChannelIdFrontRightCenter,
			ChannelId::BackCenter => raw::SoundIoChannelId::SoundIoChannelIdBackCenter,
			ChannelId::SideLeft => raw::SoundIoChannelId::SoundIoChannelIdSideLeft,
			ChannelId::SideRight => raw::SoundIoChannelId::SoundIoChannelIdSideRight,
			ChannelId::TopCenter => raw::SoundIoChannelId::SoundIoChannelIdTopCenter,
			ChannelId::TopFrontLeft => raw::SoundIoChannelId::SoundIoChannelIdTopFrontLeft,
			ChannelId::TopFrontCenter => raw::SoundIoChannelId::SoundIoChannelIdTopFrontCenter,
			ChannelId::TopFrontRight => raw::SoundIoChannelId::SoundIoChannelIdTopFrontRight,
			ChannelId::TopBackLeft => raw::SoundIoChannelId::SoundIoChannelIdTopBackLeft,
			ChannelId::TopBackCenter => raw::SoundIoChannelId::SoundIoChannelIdTopBackCenter,
			ChannelId::TopBackRight  => raw::SoundIoChannelId::SoundIoChannelIdTopBackRight ,
			ChannelId::BackLeftCenter => raw::SoundIoChannelId::SoundIoChannelIdBackLeftCenter,
			ChannelId::BackRightCenter => raw::SoundIoChannelId::SoundIoChannelIdBackRightCenter,
			ChannelId::FrontLeftWide => raw::SoundIoChannelId::SoundIoChannelIdFrontLeftWide,
			ChannelId::FrontRightWide => raw::SoundIoChannelId::SoundIoChannelIdFrontRightWide,
			ChannelId::FrontLeftHigh => raw::SoundIoChannelId::SoundIoChannelIdFrontLeftHigh,
			ChannelId::FrontCenterHigh => raw::SoundIoChannelId::SoundIoChannelIdFrontCenterHigh,
			ChannelId::FrontRightHigh => raw::SoundIoChannelId::SoundIoChannelIdFrontRightHigh,
			ChannelId::TopFrontLeftCenter => raw::SoundIoChannelId::SoundIoChannelIdTopFrontLeftCenter,
			ChannelId::TopFrontRightCenter => raw::SoundIoChannelId::SoundIoChannelIdTopFrontRightCenter,
			ChannelId::TopSideLeft => raw::SoundIoChannelId::SoundIoChannelIdTopSideLeft,
			ChannelId::TopSideRight => raw::SoundIoChannelId::SoundIoChannelIdTopSideRight,
			ChannelId::LeftLfe => raw::SoundIoChannelId::SoundIoChannelIdLeftLfe,
			ChannelId::RightLfe => raw::SoundIoChannelId::SoundIoChannelIdRightLfe,
			ChannelId::Lfe2 => raw::SoundIoChannelId::SoundIoChannelIdLfe2,
			ChannelId::BottomCenter => raw::SoundIoChannelId::SoundIoChannelIdBottomCenter,
			ChannelId::BottomLeftCenter => raw::SoundIoChannelId::SoundIoChannelIdBottomLeftCenter,
			ChannelId::BottomRightCenter => raw::SoundIoChannelId::SoundIoChannelIdBottomRightCenter,
			ChannelId::MsMid => raw::SoundIoChannelId::SoundIoChannelIdMsMid,
			ChannelId::MsSide => raw::SoundIoChannelId::SoundIoChannelIdMsSide,
			ChannelId::AmbisonicW => raw::SoundIoChannelId::SoundIoChannelIdAmbisonicW,
			ChannelId::AmbisonicX => raw::SoundIoChannelId::SoundIoChannelIdAmbisonicX,
			ChannelId::AmbisonicY => raw::SoundIoChannelId::SoundIoChannelIdAmbisonicY,
			ChannelId::AmbisonicZ => raw::SoundIoChannelId::SoundIoChannelIdAmbisonicZ,
			ChannelId::XyX => raw::SoundIoChannelId::SoundIoChannelIdXyX,
			ChannelId::XyY => raw::SoundIoChannelId::SoundIoChannelIdXyY,
			ChannelId::HeadphonesLeft => raw::SoundIoChannelId::SoundIoChannelIdHeadphonesLeft,
			ChannelId::HeadphonesRight => raw::SoundIoChannelId::SoundIoChannelIdHeadphonesRight,
			ChannelId::ClickTrack => raw::SoundIoChannelId::SoundIoChannelIdClickTrack,
			ChannelId::ForeignLanguage => raw::SoundIoChannelId::SoundIoChannelIdForeignLanguage,
			ChannelId::HearingImpaired => raw::SoundIoChannelId::SoundIoChannelIdHearingImpaired,
			ChannelId::Narration => raw::SoundIoChannelId::SoundIoChannelIdNarration,
			ChannelId::Haptic => raw::SoundIoChannelId::SoundIoChannelIdHaptic,
			ChannelId::DialogCentricMix  => raw::SoundIoChannelId::SoundIoChannelIdDialogCentricMix ,
			ChannelId::Aux => raw::SoundIoChannelId::SoundIoChannelIdAux,
			ChannelId::Aux0 => raw::SoundIoChannelId::SoundIoChannelIdAux0,
			ChannelId::Aux1 => raw::SoundIoChannelId::SoundIoChannelIdAux1,
			ChannelId::Aux2 => raw::SoundIoChannelId::SoundIoChannelIdAux2,
			ChannelId::Aux3 => raw::SoundIoChannelId::SoundIoChannelIdAux3,
			ChannelId::Aux4 => raw::SoundIoChannelId::SoundIoChannelIdAux4,
			ChannelId::Aux5 => raw::SoundIoChannelId::SoundIoChannelIdAux5,
			ChannelId::Aux6 => raw::SoundIoChannelId::SoundIoChannelIdAux6,
			ChannelId::Aux7 => raw::SoundIoChannelId::SoundIoChannelIdAux7,
			ChannelId::Aux8 => raw::SoundIoChannelId::SoundIoChannelIdAux8,
			ChannelId::Aux9 => raw::SoundIoChannelId::SoundIoChannelIdAux9,
			ChannelId::Aux10 => raw::SoundIoChannelId::SoundIoChannelIdAux10,
			ChannelId::Aux11 => raw::SoundIoChannelId::SoundIoChannelIdAux11,
			ChannelId::Aux12 => raw::SoundIoChannelId::SoundIoChannelIdAux12,
			ChannelId::Aux13 => raw::SoundIoChannelId::SoundIoChannelIdAux13,
			ChannelId::Aux14 => raw::SoundIoChannelId::SoundIoChannelIdAux14,
			ChannelId::Aux15 => raw::SoundIoChannelId::SoundIoChannelIdAux15,
			_ => raw::SoundIoChannelId::SoundIoChannelIdInvalid,
		}
    }
}


impl fmt::Display for ChannelId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let c_str: &CStr = unsafe { CStr::from_ptr(raw::soundio_get_channel_name((*self).into())) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		
		use std::error::Error;
		f.write_str(c_str.to_str().unwrap())
	}
}


/// Built-in channel layouts for convenience.
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
		
		use std::error::Error;
		f.write_str(c_str.to_str().unwrap())
	}
}

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

/// For your convenience, Native Endian and Foreign Endian constants are defined
/// which point to the respective SoundIoFormat values.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Format {
	Invalid,
	S8,        // Signed 8 bit
	U8,        // Unsigned 8 bit
	S16LE,     // Signed 16 bit Little Endian
	S16BE,     // Signed 16 bit Big Endian
	U16LE,     // Unsigned 16 bit Little Endian
	U16BE,     // Unsigned 16 bit Little Endian
	S24LE,     // Signed 24 bit Little Endian using low three bytes in 32-bit word
	S24BE,     // Signed 24 bit Big Endian using low three bytes in 32-bit word
	U24LE,     // Unsigned 24 bit Little Endian using low three bytes in 32-bit word
	U24BE,     // Unsigned 24 bit Big Endian using low three bytes in 32-bit word
	S32LE,     // Signed 32 bit Little Endian
	S32BE,     // Signed 32 bit Big Endian
	U32LE,     // Unsigned 32 bit Little Endian
	U32BE,     // Unsigned 32 bit Big Endian
	Float32LE, // Float 32 bit Little Endian, Range -1.0 to 1.0
	Float32BE, // Float 32 bit Big Endian, Range -1.0 to 1.0
	Float64LE, // Float 64 bit Little Endian, Range -1.0 to 1.0
	Float64BE, // Float 64 bit Big Endian, Range -1.0 to 1.0
}

// TODO: Add functions / constants for native / foreign endian. 

impl From<raw::SoundIoFormat> for Format {
    fn from(format: raw::SoundIoFormat) -> Format {
		match format {
			raw::SoundIoFormat::SoundIoFormatS8 => Format::S8,
			raw::SoundIoFormat::SoundIoFormatU8 => Format::U8,
			raw::SoundIoFormat::SoundIoFormatS16LE => Format::S16LE,
			raw::SoundIoFormat::SoundIoFormatS16BE => Format::S16BE,
			raw::SoundIoFormat::SoundIoFormatU16LE => Format::U16LE,
			raw::SoundIoFormat::SoundIoFormatU16BE => Format::U16BE,
			raw::SoundIoFormat::SoundIoFormatS24LE => Format::S24LE,
			raw::SoundIoFormat::SoundIoFormatS24BE => Format::S24BE,
			raw::SoundIoFormat::SoundIoFormatU24LE => Format::U24LE,
			raw::SoundIoFormat::SoundIoFormatU24BE => Format::U24BE,
			raw::SoundIoFormat::SoundIoFormatS32LE => Format::S32LE,
			raw::SoundIoFormat::SoundIoFormatS32BE => Format::S32BE,
			raw::SoundIoFormat::SoundIoFormatU32LE => Format::U32LE,
			raw::SoundIoFormat::SoundIoFormatU32BE => Format::U32BE,
			raw::SoundIoFormat::SoundIoFormatFloat32LE => Format::Float32LE,
			raw::SoundIoFormat::SoundIoFormatFloat32BE => Format::Float32BE,
			raw::SoundIoFormat::SoundIoFormatFloat64LE => Format::Float64LE,
			raw::SoundIoFormat::SoundIoFormatFloat64BE => Format::Float64BE,
			_ => Format::Invalid,
		}
    }
}

impl From<Format> for raw::SoundIoFormat {
    fn from(format: Format) -> raw::SoundIoFormat {
		match format {
			Format::S8 => raw::SoundIoFormat::SoundIoFormatS8,
			Format::U8 => raw::SoundIoFormat::SoundIoFormatU8,
			Format::S16LE => raw::SoundIoFormat::SoundIoFormatS16LE,
			Format::S16BE => raw::SoundIoFormat::SoundIoFormatS16BE,
			Format::U16LE => raw::SoundIoFormat::SoundIoFormatU16LE,
			Format::U16BE => raw::SoundIoFormat::SoundIoFormatU16BE,
			Format::S24LE => raw::SoundIoFormat::SoundIoFormatS24LE,
			Format::S24BE => raw::SoundIoFormat::SoundIoFormatS24BE,
			Format::U24LE => raw::SoundIoFormat::SoundIoFormatU24LE,
			Format::U24BE => raw::SoundIoFormat::SoundIoFormatU24BE,
			Format::S32LE => raw::SoundIoFormat::SoundIoFormatS32LE,
			Format::S32BE => raw::SoundIoFormat::SoundIoFormatS32BE,
			Format::U32LE => raw::SoundIoFormat::SoundIoFormatU32LE,
			Format::U32BE => raw::SoundIoFormat::SoundIoFormatU32BE,
			Format::Float32LE => raw::SoundIoFormat::SoundIoFormatFloat32LE,
			Format::Float32BE => raw::SoundIoFormat::SoundIoFormatFloat32BE,
			Format::Float64LE => raw::SoundIoFormat::SoundIoFormatFloat64LE,
			Format::Float64BE => raw::SoundIoFormat::SoundIoFormatFloat64BE,
			_ => raw::SoundIoFormat::SoundIoFormatInvalid,
		}
    }
}

impl Format {
	pub fn bytes_per_sample(&self) -> usize {
		unsafe { raw::soundio_get_bytes_per_sample((*self).into()) as usize }
	}
	pub fn bytes_per_frame(&self, channel_count: usize) -> usize {
		self.bytes_per_sample() * channel_count
	}
	pub fn bytes_per_second(&self, channel_count: usize, sample_rate: usize) -> usize {
		self.bytes_per_sample() * channel_count * sample_rate
	}
}

// Bit of a hack, but useful!
macro_rules! format_from_type {
    ($basic_type:ty, $format_type:path) => (
        impl From<$basic_type> for Format {
			fn from(val: $basic_type) -> Format {
				$format_type
    		}
		}
	)
}

// I'm assuming little endian. If you're using big endian I pity you.
format_from_type!(i8, Format::S8);
format_from_type!(u8, Format::U8);
format_from_type!(i16, Format::S16LE);
format_from_type!(u16, Format::U16LE);
// TODO: What about i24?
format_from_type!(i32, Format::S32LE);
format_from_type!(u32, Format::U32LE);
format_from_type!(f32, Format::Float32LE);
format_from_type!(f64, Format::Float64LE);

pub trait CastF64 {
    fn from_f64(v: f64) -> Self;
	fn to_f64(v: Self) -> f64;
}

macro_rules! impl_cast_f64 {
    ($($ty:ty)*) => {
        $(
            impl CastF64 for $ty {
                #[inline]
                fn from_f64(v: f64) -> $ty {
                    v as $ty
                }

                #[inline]
                fn to_f64(v: $ty) -> f64 {
                    v as f64
                }
            }
        )*
    }
}

impl_cast_f64!(u8 u16 u32 u64 usize i8 i16 i32 i64 isize f32 f64);

impl fmt::Display for Format {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let c_str: &CStr = unsafe { CStr::from_ptr(raw::soundio_format_string((*self).into())) };

		// TODO: to_str() checks for valid UTF-8 since that what a &str is. Is it safe to assume
		// soundio_strerror() never returns invalid UTF-8?
		
		use std::error::Error;
		f.write_str(c_str.to_str().unwrap())
	}
}



// TODO: Just use the standard range objects?
#[derive(Debug, Copy, Clone)]
pub struct SampleRateRange {
	pub min: i32,
	pub max: i32,
}