#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use bindings;

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

impl From<bindings::SoundIoChannelId> for ChannelId {
    fn from(channel_id: bindings::SoundIoChannelId) -> ChannelId {
		match channel_id {
			bindings::SoundIoChannelId::SoundIoChannelIdFrontLeft => ChannelId::FrontLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontRight => ChannelId::FrontRight,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontCenter => ChannelId::FrontCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdLfe => ChannelId::Lfe,
			bindings::SoundIoChannelId::SoundIoChannelIdBackLeft => ChannelId::BackLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdBackRight => ChannelId::BackRight,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontLeftCenter => ChannelId::FrontLeftCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontRightCenter => ChannelId::FrontRightCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdBackCenter => ChannelId::BackCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdSideLeft => ChannelId::SideLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdSideRight => ChannelId::SideRight,
			bindings::SoundIoChannelId::SoundIoChannelIdTopCenter => ChannelId::TopCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdTopFrontLeft => ChannelId::TopFrontLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdTopFrontCenter => ChannelId::TopFrontCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdTopFrontRight => ChannelId::TopFrontRight,
			bindings::SoundIoChannelId::SoundIoChannelIdTopBackLeft => ChannelId::TopBackLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdTopBackCenter => ChannelId::TopBackCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdTopBackRight  => ChannelId::TopBackRight ,
			bindings::SoundIoChannelId::SoundIoChannelIdBackLeftCenter => ChannelId::BackLeftCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdBackRightCenter => ChannelId::BackRightCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontLeftWide => ChannelId::FrontLeftWide,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontRightWide => ChannelId::FrontRightWide,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontLeftHigh => ChannelId::FrontLeftHigh,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontCenterHigh => ChannelId::FrontCenterHigh,
			bindings::SoundIoChannelId::SoundIoChannelIdFrontRightHigh => ChannelId::FrontRightHigh,
			bindings::SoundIoChannelId::SoundIoChannelIdTopFrontLeftCenter => ChannelId::TopFrontLeftCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdTopFrontRightCenter => ChannelId::TopFrontRightCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdTopSideLeft => ChannelId::TopSideLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdTopSideRight => ChannelId::TopSideRight,
			bindings::SoundIoChannelId::SoundIoChannelIdLeftLfe => ChannelId::LeftLfe,
			bindings::SoundIoChannelId::SoundIoChannelIdRightLfe => ChannelId::RightLfe,
			bindings::SoundIoChannelId::SoundIoChannelIdLfe2 => ChannelId::Lfe2,
			bindings::SoundIoChannelId::SoundIoChannelIdBottomCenter => ChannelId::BottomCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdBottomLeftCenter => ChannelId::BottomLeftCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdBottomRightCenter => ChannelId::BottomRightCenter,
			bindings::SoundIoChannelId::SoundIoChannelIdMsMid => ChannelId::MsMid,
			bindings::SoundIoChannelId::SoundIoChannelIdMsSide => ChannelId::MsSide,
			bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicW => ChannelId::AmbisonicW,
			bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicX => ChannelId::AmbisonicX,
			bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicY => ChannelId::AmbisonicY,
			bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicZ => ChannelId::AmbisonicZ,
			bindings::SoundIoChannelId::SoundIoChannelIdXyX => ChannelId::XyX,
			bindings::SoundIoChannelId::SoundIoChannelIdXyY => ChannelId::XyY,
			bindings::SoundIoChannelId::SoundIoChannelIdHeadphonesLeft => ChannelId::HeadphonesLeft,
			bindings::SoundIoChannelId::SoundIoChannelIdHeadphonesRight => ChannelId::HeadphonesRight,
			bindings::SoundIoChannelId::SoundIoChannelIdClickTrack => ChannelId::ClickTrack,
			bindings::SoundIoChannelId::SoundIoChannelIdForeignLanguage => ChannelId::ForeignLanguage,
			bindings::SoundIoChannelId::SoundIoChannelIdHearingImpaired => ChannelId::HearingImpaired,
			bindings::SoundIoChannelId::SoundIoChannelIdNarration => ChannelId::Narration,
			bindings::SoundIoChannelId::SoundIoChannelIdHaptic => ChannelId::Haptic,
			bindings::SoundIoChannelId::SoundIoChannelIdDialogCentricMix  => ChannelId::DialogCentricMix ,
			bindings::SoundIoChannelId::SoundIoChannelIdAux => ChannelId::Aux,
			bindings::SoundIoChannelId::SoundIoChannelIdAux0 => ChannelId::Aux0,
			bindings::SoundIoChannelId::SoundIoChannelIdAux1 => ChannelId::Aux1,
			bindings::SoundIoChannelId::SoundIoChannelIdAux2 => ChannelId::Aux2,
			bindings::SoundIoChannelId::SoundIoChannelIdAux3 => ChannelId::Aux3,
			bindings::SoundIoChannelId::SoundIoChannelIdAux4 => ChannelId::Aux4,
			bindings::SoundIoChannelId::SoundIoChannelIdAux5 => ChannelId::Aux5,
			bindings::SoundIoChannelId::SoundIoChannelIdAux6 => ChannelId::Aux6,
			bindings::SoundIoChannelId::SoundIoChannelIdAux7 => ChannelId::Aux7,
			bindings::SoundIoChannelId::SoundIoChannelIdAux8 => ChannelId::Aux8,
			bindings::SoundIoChannelId::SoundIoChannelIdAux9 => ChannelId::Aux9,
			bindings::SoundIoChannelId::SoundIoChannelIdAux10 => ChannelId::Aux10,
			bindings::SoundIoChannelId::SoundIoChannelIdAux11 => ChannelId::Aux11,
			bindings::SoundIoChannelId::SoundIoChannelIdAux12 => ChannelId::Aux12,
			bindings::SoundIoChannelId::SoundIoChannelIdAux13 => ChannelId::Aux13,
			bindings::SoundIoChannelId::SoundIoChannelIdAux14 => ChannelId::Aux14,
			bindings::SoundIoChannelId::SoundIoChannelIdAux15 => ChannelId::Aux15,
			_ => ChannelId::Invalid,
		}
    }
}

impl From<ChannelId> for bindings::SoundIoChannelId {
    fn from(channel_id: ChannelId) -> bindings::SoundIoChannelId {
		match channel_id {
			ChannelId::FrontLeft => bindings::SoundIoChannelId::SoundIoChannelIdFrontLeft,
			ChannelId::FrontRight => bindings::SoundIoChannelId::SoundIoChannelIdFrontRight,
			ChannelId::FrontCenter => bindings::SoundIoChannelId::SoundIoChannelIdFrontCenter,
			ChannelId::Lfe => bindings::SoundIoChannelId::SoundIoChannelIdLfe,
			ChannelId::BackLeft => bindings::SoundIoChannelId::SoundIoChannelIdBackLeft,
			ChannelId::BackRight => bindings::SoundIoChannelId::SoundIoChannelIdBackRight,
			ChannelId::FrontLeftCenter => bindings::SoundIoChannelId::SoundIoChannelIdFrontLeftCenter,
			ChannelId::FrontRightCenter => bindings::SoundIoChannelId::SoundIoChannelIdFrontRightCenter,
			ChannelId::BackCenter => bindings::SoundIoChannelId::SoundIoChannelIdBackCenter,
			ChannelId::SideLeft => bindings::SoundIoChannelId::SoundIoChannelIdSideLeft,
			ChannelId::SideRight => bindings::SoundIoChannelId::SoundIoChannelIdSideRight,
			ChannelId::TopCenter => bindings::SoundIoChannelId::SoundIoChannelIdTopCenter,
			ChannelId::TopFrontLeft => bindings::SoundIoChannelId::SoundIoChannelIdTopFrontLeft,
			ChannelId::TopFrontCenter => bindings::SoundIoChannelId::SoundIoChannelIdTopFrontCenter,
			ChannelId::TopFrontRight => bindings::SoundIoChannelId::SoundIoChannelIdTopFrontRight,
			ChannelId::TopBackLeft => bindings::SoundIoChannelId::SoundIoChannelIdTopBackLeft,
			ChannelId::TopBackCenter => bindings::SoundIoChannelId::SoundIoChannelIdTopBackCenter,
			ChannelId::TopBackRight  => bindings::SoundIoChannelId::SoundIoChannelIdTopBackRight ,
			ChannelId::BackLeftCenter => bindings::SoundIoChannelId::SoundIoChannelIdBackLeftCenter,
			ChannelId::BackRightCenter => bindings::SoundIoChannelId::SoundIoChannelIdBackRightCenter,
			ChannelId::FrontLeftWide => bindings::SoundIoChannelId::SoundIoChannelIdFrontLeftWide,
			ChannelId::FrontRightWide => bindings::SoundIoChannelId::SoundIoChannelIdFrontRightWide,
			ChannelId::FrontLeftHigh => bindings::SoundIoChannelId::SoundIoChannelIdFrontLeftHigh,
			ChannelId::FrontCenterHigh => bindings::SoundIoChannelId::SoundIoChannelIdFrontCenterHigh,
			ChannelId::FrontRightHigh => bindings::SoundIoChannelId::SoundIoChannelIdFrontRightHigh,
			ChannelId::TopFrontLeftCenter => bindings::SoundIoChannelId::SoundIoChannelIdTopFrontLeftCenter,
			ChannelId::TopFrontRightCenter => bindings::SoundIoChannelId::SoundIoChannelIdTopFrontRightCenter,
			ChannelId::TopSideLeft => bindings::SoundIoChannelId::SoundIoChannelIdTopSideLeft,
			ChannelId::TopSideRight => bindings::SoundIoChannelId::SoundIoChannelIdTopSideRight,
			ChannelId::LeftLfe => bindings::SoundIoChannelId::SoundIoChannelIdLeftLfe,
			ChannelId::RightLfe => bindings::SoundIoChannelId::SoundIoChannelIdRightLfe,
			ChannelId::Lfe2 => bindings::SoundIoChannelId::SoundIoChannelIdLfe2,
			ChannelId::BottomCenter => bindings::SoundIoChannelId::SoundIoChannelIdBottomCenter,
			ChannelId::BottomLeftCenter => bindings::SoundIoChannelId::SoundIoChannelIdBottomLeftCenter,
			ChannelId::BottomRightCenter => bindings::SoundIoChannelId::SoundIoChannelIdBottomRightCenter,
			ChannelId::MsMid => bindings::SoundIoChannelId::SoundIoChannelIdMsMid,
			ChannelId::MsSide => bindings::SoundIoChannelId::SoundIoChannelIdMsSide,
			ChannelId::AmbisonicW => bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicW,
			ChannelId::AmbisonicX => bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicX,
			ChannelId::AmbisonicY => bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicY,
			ChannelId::AmbisonicZ => bindings::SoundIoChannelId::SoundIoChannelIdAmbisonicZ,
			ChannelId::XyX => bindings::SoundIoChannelId::SoundIoChannelIdXyX,
			ChannelId::XyY => bindings::SoundIoChannelId::SoundIoChannelIdXyY,
			ChannelId::HeadphonesLeft => bindings::SoundIoChannelId::SoundIoChannelIdHeadphonesLeft,
			ChannelId::HeadphonesRight => bindings::SoundIoChannelId::SoundIoChannelIdHeadphonesRight,
			ChannelId::ClickTrack => bindings::SoundIoChannelId::SoundIoChannelIdClickTrack,
			ChannelId::ForeignLanguage => bindings::SoundIoChannelId::SoundIoChannelIdForeignLanguage,
			ChannelId::HearingImpaired => bindings::SoundIoChannelId::SoundIoChannelIdHearingImpaired,
			ChannelId::Narration => bindings::SoundIoChannelId::SoundIoChannelIdNarration,
			ChannelId::Haptic => bindings::SoundIoChannelId::SoundIoChannelIdHaptic,
			ChannelId::DialogCentricMix  => bindings::SoundIoChannelId::SoundIoChannelIdDialogCentricMix ,
			ChannelId::Aux => bindings::SoundIoChannelId::SoundIoChannelIdAux,
			ChannelId::Aux0 => bindings::SoundIoChannelId::SoundIoChannelIdAux0,
			ChannelId::Aux1 => bindings::SoundIoChannelId::SoundIoChannelIdAux1,
			ChannelId::Aux2 => bindings::SoundIoChannelId::SoundIoChannelIdAux2,
			ChannelId::Aux3 => bindings::SoundIoChannelId::SoundIoChannelIdAux3,
			ChannelId::Aux4 => bindings::SoundIoChannelId::SoundIoChannelIdAux4,
			ChannelId::Aux5 => bindings::SoundIoChannelId::SoundIoChannelIdAux5,
			ChannelId::Aux6 => bindings::SoundIoChannelId::SoundIoChannelIdAux6,
			ChannelId::Aux7 => bindings::SoundIoChannelId::SoundIoChannelIdAux7,
			ChannelId::Aux8 => bindings::SoundIoChannelId::SoundIoChannelIdAux8,
			ChannelId::Aux9 => bindings::SoundIoChannelId::SoundIoChannelIdAux9,
			ChannelId::Aux10 => bindings::SoundIoChannelId::SoundIoChannelIdAux10,
			ChannelId::Aux11 => bindings::SoundIoChannelId::SoundIoChannelIdAux11,
			ChannelId::Aux12 => bindings::SoundIoChannelId::SoundIoChannelIdAux12,
			ChannelId::Aux13 => bindings::SoundIoChannelId::SoundIoChannelIdAux13,
			ChannelId::Aux14 => bindings::SoundIoChannelId::SoundIoChannelIdAux14,
			ChannelId::Aux15 => bindings::SoundIoChannelId::SoundIoChannelIdAux15,
			_ => bindings::SoundIoChannelId::SoundIoChannelIdInvalid,
		}
    }
}


impl fmt::Display for ChannelId {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let c_str: &CStr = unsafe { CStr::from_ptr(bindings::soundio_get_channel_name((*self).into())) };

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


impl From<bindings::SoundIoChannelLayoutId> for ChannelLayoutId {
    fn from(channel_layout_id: bindings::SoundIoChannelLayoutId) -> ChannelLayoutId {
		match channel_layout_id {
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdMono            => ChannelLayoutId::Mono,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdStereo          => ChannelLayoutId::Stereo,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId2Point1         => ChannelLayoutId::C2Point1,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0         => ChannelLayoutId::C3Point0,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0Back     => ChannelLayoutId::C3Point0Back,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point1         => ChannelLayoutId::C3Point1,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point0         => ChannelLayoutId::C4Point0,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuad            => ChannelLayoutId::Quad,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuadSide        => ChannelLayoutId::QuadSide,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point1         => ChannelLayoutId::C4Point1,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Back     => ChannelLayoutId::C5Point0Back,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Side     => ChannelLayoutId::C5Point0Side,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1         => ChannelLayoutId::C5Point1,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1Back     => ChannelLayoutId::C5Point1Back,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Side     => ChannelLayoutId::C6Point0Side,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Front    => ChannelLayoutId::C6Point0Front,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdHexagonal       => ChannelLayoutId::Hexagonal,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1         => ChannelLayoutId::C6Point1,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Back     => ChannelLayoutId::C6Point1Back,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Front    => ChannelLayoutId::C6Point1Front,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0         => ChannelLayoutId::C7Point0,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0Front    => ChannelLayoutId::C7Point0Front,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1         => ChannelLayoutId::C7Point1,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1Wide     => ChannelLayoutId::C7Point1Wide,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1WideBack => ChannelLayoutId::C7Point1WideBack,
			bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdOctagonal       => ChannelLayoutId::Octagonal,
		}
    }
}

impl From<ChannelLayoutId> for bindings::SoundIoChannelLayoutId {
    fn from(channel_layout_id: ChannelLayoutId) -> bindings::SoundIoChannelLayoutId {
		match channel_layout_id {
			ChannelLayoutId::Mono             => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdMono,
			ChannelLayoutId::Stereo           => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdStereo,
			ChannelLayoutId::C2Point1         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId2Point1,
			ChannelLayoutId::C3Point0         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0,
			ChannelLayoutId::C3Point0Back     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point0Back,
			ChannelLayoutId::C3Point1         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId3Point1,
			ChannelLayoutId::C4Point0         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point0,
			ChannelLayoutId::Quad             => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuad,
			ChannelLayoutId::QuadSide         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdQuadSide,
			ChannelLayoutId::C4Point1         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId4Point1,
			ChannelLayoutId::C5Point0Back     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Back,
			ChannelLayoutId::C5Point0Side     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point0Side,
			ChannelLayoutId::C5Point1         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1,
			ChannelLayoutId::C5Point1Back     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId5Point1Back,
			ChannelLayoutId::C6Point0Side     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Side,
			ChannelLayoutId::C6Point0Front    => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point0Front,
			ChannelLayoutId::Hexagonal        => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdHexagonal,
			ChannelLayoutId::C6Point1         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1,
			ChannelLayoutId::C6Point1Back     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Back,
			ChannelLayoutId::C6Point1Front    => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId6Point1Front,
			ChannelLayoutId::C7Point0         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0,
			ChannelLayoutId::C7Point0Front    => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point0Front,
			ChannelLayoutId::C7Point1         => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1,
			ChannelLayoutId::C7Point1Wide     => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1Wide,
			ChannelLayoutId::C7Point1WideBack => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutId7Point1WideBack,
			ChannelLayoutId::Octagonal        => bindings::SoundIoChannelLayoutId::SoundIoChannelLayoutIdOctagonal,
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

impl From<bindings::SoundIoBackend> for Backend {
    fn from(backend: bindings::SoundIoBackend) -> Backend {
		match backend {
			bindings::SoundIoBackend::SoundIoBackendJack => Backend::Jack,
			bindings::SoundIoBackend::SoundIoBackendPulseAudio => Backend::PulseAudio,
			bindings::SoundIoBackend::SoundIoBackendAlsa => Backend::Alsa,
			bindings::SoundIoBackend::SoundIoBackendCoreAudio => Backend::CoreAudio,
			bindings::SoundIoBackend::SoundIoBackendWasapi => Backend::Wasapi,
			bindings::SoundIoBackend::SoundIoBackendDummy => Backend::Dummy,
			_ => Backend::None,
		}
    }
}

impl From<Backend> for bindings::SoundIoBackend {
    fn from(backend: Backend) -> bindings::SoundIoBackend {
		match backend {
			Backend::Jack => bindings::SoundIoBackend::SoundIoBackendJack,
			Backend::PulseAudio => bindings::SoundIoBackend::SoundIoBackendPulseAudio,
			Backend::Alsa => bindings::SoundIoBackend::SoundIoBackendAlsa,
			Backend::CoreAudio => bindings::SoundIoBackend::SoundIoBackendCoreAudio,
			Backend::Wasapi => bindings::SoundIoBackend::SoundIoBackendWasapi,
			Backend::Dummy => bindings::SoundIoBackend::SoundIoBackendDummy,
			_ => bindings::SoundIoBackend::SoundIoBackendNone,
		}
    }
}

impl fmt::Display for Backend {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// TODO: This may be overkill - I could just use the #[derive(Debug)] output; it's nearly identical.

		let c_str: &CStr = unsafe { CStr::from_ptr(bindings::soundio_backend_name((*self).into())) };

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

impl From<bindings::SoundIoDeviceAim> for DeviceAim {
    fn from(aim: bindings::SoundIoDeviceAim) -> DeviceAim {
		match aim {
			bindings::SoundIoDeviceAim::SoundIoDeviceAimInput => DeviceAim::Input,
			bindings::SoundIoDeviceAim::SoundIoDeviceAimOutput => DeviceAim::Output,
		}
    }
}

impl From<DeviceAim> for bindings::SoundIoDeviceAim {
    fn from(aim: DeviceAim) -> bindings::SoundIoDeviceAim {
		match aim {
			DeviceAim::Input => bindings::SoundIoDeviceAim::SoundIoDeviceAimInput,
			DeviceAim::Output => bindings::SoundIoDeviceAim::SoundIoDeviceAimOutput,
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

impl From<bindings::SoundIoFormat> for Format {
    fn from(format: bindings::SoundIoFormat) -> Format {
		match format {
			bindings::SoundIoFormat::SoundIoFormatS8 => Format::S8,
			bindings::SoundIoFormat::SoundIoFormatU8 => Format::U8,
			bindings::SoundIoFormat::SoundIoFormatS16LE => Format::S16LE,
			bindings::SoundIoFormat::SoundIoFormatS16BE => Format::S16BE,
			bindings::SoundIoFormat::SoundIoFormatU16LE => Format::U16LE,
			bindings::SoundIoFormat::SoundIoFormatU16BE => Format::U16BE,
			bindings::SoundIoFormat::SoundIoFormatS24LE => Format::S24LE,
			bindings::SoundIoFormat::SoundIoFormatS24BE => Format::S24BE,
			bindings::SoundIoFormat::SoundIoFormatU24LE => Format::U24LE,
			bindings::SoundIoFormat::SoundIoFormatU24BE => Format::U24BE,
			bindings::SoundIoFormat::SoundIoFormatS32LE => Format::S32LE,
			bindings::SoundIoFormat::SoundIoFormatS32BE => Format::S32BE,
			bindings::SoundIoFormat::SoundIoFormatU32LE => Format::U32LE,
			bindings::SoundIoFormat::SoundIoFormatU32BE => Format::U32BE,
			bindings::SoundIoFormat::SoundIoFormatFloat32LE => Format::Float32LE,
			bindings::SoundIoFormat::SoundIoFormatFloat32BE => Format::Float32BE,
			bindings::SoundIoFormat::SoundIoFormatFloat64LE => Format::Float64LE,
			bindings::SoundIoFormat::SoundIoFormatFloat64BE => Format::Float64BE,
			_ => Format::Invalid,
		}
    }
}

impl From<Format> for bindings::SoundIoFormat {
    fn from(format: Format) -> bindings::SoundIoFormat {
		match format {
			Format::S8 => bindings::SoundIoFormat::SoundIoFormatS8,
			Format::U8 => bindings::SoundIoFormat::SoundIoFormatU8,
			Format::S16LE => bindings::SoundIoFormat::SoundIoFormatS16LE,
			Format::S16BE => bindings::SoundIoFormat::SoundIoFormatS16BE,
			Format::U16LE => bindings::SoundIoFormat::SoundIoFormatU16LE,
			Format::U16BE => bindings::SoundIoFormat::SoundIoFormatU16BE,
			Format::S24LE => bindings::SoundIoFormat::SoundIoFormatS24LE,
			Format::S24BE => bindings::SoundIoFormat::SoundIoFormatS24BE,
			Format::U24LE => bindings::SoundIoFormat::SoundIoFormatU24LE,
			Format::U24BE => bindings::SoundIoFormat::SoundIoFormatU24BE,
			Format::S32LE => bindings::SoundIoFormat::SoundIoFormatS32LE,
			Format::S32BE => bindings::SoundIoFormat::SoundIoFormatS32BE,
			Format::U32LE => bindings::SoundIoFormat::SoundIoFormatU32LE,
			Format::U32BE => bindings::SoundIoFormat::SoundIoFormatU32BE,
			Format::Float32LE => bindings::SoundIoFormat::SoundIoFormatFloat32LE,
			Format::Float32BE => bindings::SoundIoFormat::SoundIoFormatFloat32BE,
			Format::Float64LE => bindings::SoundIoFormat::SoundIoFormatFloat64LE,
			Format::Float64BE => bindings::SoundIoFormat::SoundIoFormatFloat64BE,
			_ => bindings::SoundIoFormat::SoundIoFormatInvalid,
		}
    }
}

impl Format {
	pub fn bytes_per_sample(&self) -> i32 {
		unsafe { bindings::soundio_get_bytes_per_sample((*self).into()) as i32 }
	}
	pub fn bytes_per_frame(&self, channel_count: i32) -> i32 {
		self.bytes_per_sample() * channel_count
	}
	pub fn bytes_per_second(&self, channel_count: i32, sample_rate: i32) -> i32 {
		self.bytes_per_sample() * channel_count * sample_rate
	}
}

impl fmt::Display for Format {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let c_str: &CStr = unsafe { CStr::from_ptr(bindings::soundio_format_string((*self).into())) };

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