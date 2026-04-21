// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the trailer field
(ANSI/VITA-49.2-2017 section 5.1.6).
*/

use deku::prelude::*;

/// Sample frame indicator enumeration.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SampleFrameIndicator {
    /// Sample Frames are not applicable to data packets, or the entire Sample
    /// Frame is contained in a single data packet
    #[deku(id = 0x0)]
    NotApplicable,
    /// First data packet of current Sample Frame
    #[deku(id = 0x1)]
    FirstDataPacket,
    /// Middle packet or packets of Sample Frame, i.e. "continuation" indicator
    #[deku(id = 0x2)]
    MiddleDataPacket,
    /// Final data packet of current Sample Frame
    #[deku(id = 0x3)]
    FinalDataPacket,
}

impl TryFrom<u32> for SampleFrameIndicator {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if x == SampleFrameIndicator::NotApplicable as u32 => {
                Ok(SampleFrameIndicator::NotApplicable)
            }
            x if x == SampleFrameIndicator::FirstDataPacket as u32 => {
                Ok(SampleFrameIndicator::FirstDataPacket)
            }
            x if x == SampleFrameIndicator::MiddleDataPacket as u32 => {
                Ok(SampleFrameIndicator::MiddleDataPacket)
            }
            x if x == SampleFrameIndicator::FinalDataPacket as u32 => {
                Ok(SampleFrameIndicator::FinalDataPacket)
            }
            _ => Err(()),
        }
    }
}

/// Base trailer field data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trailer(u32);

impl Trailer {
    fn cal_time_enabled(&self) -> bool {
        self.0 & (1 << 31) > 0
    }
    fn valid_data_enabled(&self) -> bool {
        self.0 & (1 << 30) > 0
    }
    fn reference_lock_enabled(&self) -> bool {
        self.0 & (1 << 29) > 0
    }
    fn agc_enabled(&self) -> bool {
        self.0 & (1 << 28) > 0
    }
    fn detected_signal_enabled(&self) -> bool {
        self.0 & (1 << 27) > 0
    }
    fn spectral_inversion_enabled(&self) -> bool {
        self.0 & (1 << 26) > 0
    }
    fn over_range_enabled(&self) -> bool {
        self.0 & (1 << 25) > 0
    }
    fn sample_loss_enabled(&self) -> bool {
        self.0 & (1 << 24) > 0
    }
    fn sample_frame_enabled(&self) -> bool {
        self.0 & (1 << 23) > 0 && self.0 & (1 << 22) > 0
    }
    fn user_defined_enabled(&self) -> bool {
        self.0 & (1 << 21) > 0 && self.0 & (1 << 20) > 0
    }
    /// Returns the calibration time indicator status if present.
    pub fn cal_time_indicator(&self) -> Option<bool> {
        if self.cal_time_enabled() {
            Some(self.0 & (1 << 19) > 0)
        } else {
            None
        }
    }
    /// Returns the valid data indicator status if present.
    pub fn valid_data_indicator(&self) -> Option<bool> {
        if self.valid_data_enabled() {
            Some(self.0 & (1 << 18) > 0)
        } else {
            None
        }
    }
    /// Returns the reference lock indicator status if present.
    pub fn reference_lock_indicator(&self) -> Option<bool> {
        if self.reference_lock_enabled() {
            Some(self.0 & (1 << 17) > 0)
        } else {
            None
        }
    }
    /// Returns the automcatic gain control (AGC) indicator status if present.
    pub fn agc_indicator(&self) -> Option<bool> {
        if self.agc_enabled() {
            Some(self.0 & (1 << 16) > 0)
        } else {
            None
        }
    }
    /// Returns the detected signal indicator status if present.
    pub fn detected_signal_indicator(&self) -> Option<bool> {
        if self.detected_signal_enabled() {
            Some(self.0 & (1 << 15) > 0)
        } else {
            None
        }
    }
    /// Returns the spectral inversion indicator status if present.
    pub fn spectral_inversion_indicator(&self) -> Option<bool> {
        if self.spectral_inversion_enabled() {
            Some(self.0 & (1 << 14) > 0)
        } else {
            None
        }
    }
    /// Returns the over range indicator status if present.
    pub fn over_range_indicator(&self) -> Option<bool> {
        if self.over_range_enabled() {
            Some(self.0 & (1 << 13) > 0)
        } else {
            None
        }
    }
    /// Returns the sample loss indicator status if present.
    pub fn sample_loss_indicator(&self) -> Option<bool> {
        if self.sample_loss_enabled() {
            Some(self.0 & (1 << 12) > 0)
        } else {
            None
        }
    }
    /// Returns the sample frame indicator status if present.
    pub fn sample_frame_indicator(&self) -> Option<SampleFrameIndicator> {
        if self.sample_frame_enabled() {
            Some(((self.0 >> 10) & 0b11).try_into().unwrap())
        } else {
            None
        }
    }
    /// Returns the user-defined indicator status byte if present.
    pub fn user_defined_indicator(&self) -> Option<u8> {
        if self.user_defined_enabled() {
            Some(((self.0 >> 8) & 0b11) as u8)
        } else {
            None
        }
    }
    fn associated_context_packet_count_enabled(&self) -> bool {
        self.0 & (1 << 7) > 0
    }
    /// Returns the associated context packet count if present.
    pub fn associated_context_packet_count(&self) -> Option<u8> {
        if self.associated_context_packet_count_enabled() {
            Some((self.0 & 0x7F) as u8)
        } else {
            None
        }
    }
}
