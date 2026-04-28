// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the packet header format
(ANSI/VITA-49.2-2017 section 5.1.1).
*/

use deku::prelude::*;

use crate::VitaError;

/// Base packet header data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PacketHeader {
    hword_1: u16,
    packet_size: u16,
}

/// The type of VRT packet being worked on.
///
/// Note: the packet type is used throughout this crate to determine
/// how to serialize and deserialize various fields, so it's important
/// that this field is correctly set.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PacketType {
    /// Signal data packet without a stream ID.
    #[deku(id = 0x0)]
    SignalDataWithoutStreamId,
    /// Signal data packet *with* a stream ID.
    #[deku(id = 0x1)]
    SignalData,
    /// Extension data packet without a stream ID.
    #[deku(id = 0x2)]
    ExtensionDataWithoutStreamId,
    /// Extension data packet *with* a stream ID.
    #[deku(id = 0x3)]
    ExtensionData,
    /// Context packet.
    #[deku(id = 0x4)]
    Context,
    /// Extension context packet.
    #[deku(id = 0x5)]
    ExtensionContext,
    /// Command packet.
    #[deku(id = 0x6)]
    Command,
    /// Extension command packet.
    #[deku(id = 0x7)]
    ExtensionCommand,
    // All other values are reserved
}

impl PacketType {
    /// Returns true if the packet type has a signal data-style payload.
    pub fn has_signal_data_payload(&self) -> bool {
        !matches!(
            &self,
            PacketType::SignalData
                | PacketType::ExtensionData
                | PacketType::SignalDataWithoutStreamId
                | PacketType::ExtensionDataWithoutStreamId
        )
    }
    /// Returns true if the packet type has a context-style payload.
    pub fn has_context_payload(&self) -> bool {
        !matches!(&self, PacketType::Context | PacketType::ExtensionContext)
    }
    /// Returns true if the packet type has a command-style payload.
    pub fn has_command_payload(&self) -> bool {
        !matches!(&self, PacketType::Command | PacketType::ExtensionCommand)
    }
}

impl TryFrom<u8> for PacketType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == PacketType::SignalDataWithoutStreamId as u8 => {
                Ok(PacketType::SignalDataWithoutStreamId)
            }
            x if x == PacketType::SignalData as u8 => Ok(PacketType::SignalData),
            x if x == PacketType::ExtensionDataWithoutStreamId as u8 => {
                Ok(PacketType::ExtensionDataWithoutStreamId)
            }
            x if x == PacketType::ExtensionData as u8 => Ok(PacketType::ExtensionData),
            x if x == PacketType::Context as u8 => Ok(PacketType::Context),
            x if x == PacketType::ExtensionContext as u8 => Ok(PacketType::ExtensionContext),
            x if x == PacketType::Command as u8 => Ok(PacketType::Command),
            x if x == PacketType::ExtensionCommand as u8 => Ok(PacketType::ExtensionCommand),
            _ => Err(()),
        }
    }
}

/// Indicator field enumeration. The three indicator bits
/// have different meaning depending on if the packet is a
/// signal data, context, or command packet.
///
///
/// | Header Bit | Indicator Bit | Data Packet | Context Packet | Command Packet |
/// |------------|---------------|-------------|----------------|----------------|
/// | 26 | 10 | Trailer Included | *Reserved* | Acknowledge Packet |
/// | 25 | 9  | Not a V49.0 Packet | Not a V49.0 Packet | *Reserved* |
/// | 24 | 8  | Spectrum or Time Packet | Timestamp Mode | Cancellation Packet |
///
/// See ANSI/VITA-49.2-2017 section 5.1.1.1 for more details.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, packet_type: PacketType",
    id = "packet_type"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Indicators {
    /// The bits represent signal data indicators.
    #[deku(id = "PacketType::SignalData")]
    SignalData(SignalDataIndicators),
    /// The bits represent context indicators.
    #[deku(id = "PacketType::Context")]
    Context(ContextIndicators),
    /// The bits represent command indicators.
    #[deku(id = "PacketType::Command")]
    Command(CommandIndicators),
}

impl Indicators {
    /// Offset into 2-byte word where the three bit indicators are stored
    const OFFSET: u8 = 8;
    /// Offset of left-most indicator field
    const F1: u8 = Self::OFFSET + 2;
    /// Offset of middle indicator field
    const F2: u8 = Self::OFFSET + 1;
    /// Offset of right-most indicator field
    const F3: u8 = Self::OFFSET;
    /// The bitmask for the indicator bits in the packet header.
    const MASK: u16 = 0b111 << Self::OFFSET;
    /// Gets the raw 16-bit value of the collected indicator flags.
    fn as_u16(&self) -> u16 {
        match self {
            Indicators::SignalData(i) => {
                (i.trailer_included as u16) << Self::F1
                    | (i.not_a_vita490_packet as u16) << Self::F2
                    | (i.signal_spectral_data as u16) << Self::F3
            }
            Indicators::Context(i) => {
                // F1 is reserved
                (i.not_a_vita490_packet as u16) << Self::F2 | (i.timestamp_mode as u16) << Self::F3
            }
            Indicators::Command(i) => {
                // F2 is reserved
                (i.ack_packet as u16) << Self::F1 | (i.cancellation_packet as u16) << Self::F3
            }
        }
    }
    /// Creates an Indicators struct from the raw 16-bit header value
    fn from_u16_for_packet(value: u16, p_type: PacketType) -> Self {
        let i1 = value & (1 << Self::F1) > 1;
        let i2 = value & (1 << Self::F2) > 1;
        let i3 = value & (1 << Self::F3) > 1;
        match p_type {
            PacketType::SignalData
            | PacketType::SignalDataWithoutStreamId
            | PacketType::ExtensionData
            | PacketType::ExtensionDataWithoutStreamId => {
                Indicators::SignalData(SignalDataIndicators {
                    trailer_included: i1,
                    not_a_vita490_packet: i2,
                    signal_spectral_data: i3,
                })
            }
            PacketType::Context | PacketType::ExtensionContext => {
                Indicators::Context(ContextIndicators {
                    // i1 is reserved
                    not_a_vita490_packet: i2,
                    timestamp_mode: i3.try_into().unwrap(),
                })
            }
            PacketType::Command | PacketType::ExtensionCommand => {
                Indicators::Command(CommandIndicators {
                    ack_packet: i1,
                    // i2 is reserved
                    cancellation_packet: i3,
                })
            }
        }
    }
}

/// Signal data indicator fields.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignalDataIndicators {
    /// The packet includes a trailer.
    pub trailer_included: bool,
    /// The packet is not compliant with VITA 49.0. A VITA 49.0 parser may
    /// break if trying to parse this packet.
    pub not_a_vita490_packet: bool,
    /// The signal data represents spectral data.
    pub signal_spectral_data: bool,
}

/// Timestamp mode
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimestampMode {
    /// Used to convey the precise timing of events or Context changes.
    /// The resolution of this Timestamp Mode could be up to highest
    /// resolution supported by the TSF setting (either sample or picosecond
    /// resolution).
    #[deku(id = 0x0)]
    PreciseTiming,
    // Used to convey the general timing of events or Context changes.
    /// The resolution of this Timestamp Mode is the Data Sampling Interval
    /// of a Data packet.
    #[deku(id = 0x1)]
    GeneralTiming,
}

impl TryFrom<bool> for TimestampMode {
    type Error = ();

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        if value {
            Ok(TimestampMode::GeneralTiming)
        } else {
            Ok(TimestampMode::PreciseTiming)
        }
    }
}

/// Context packet indicator fields.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContextIndicators {
    /// The packet is not compliant with VITA 49.0. A VITA 49.0 parser may
    /// break if trying to parse this packet.
    pub not_a_vita490_packet: bool,
    /// Context timestamp mode.
    pub timestamp_mode: TimestampMode,
}

/// Command packet indicators.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CommandIndicators {
    /// The command packet is an ACK packet.
    pub ack_packet: bool,
    /// The command packet is a cancellation packet.
    pub cancellation_packet: bool,
}

/// TimeStamp-Integer (TSI) field.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tsi {
    /// Timestamp is not included.
    #[deku(id = 0x0)]
    Null,
    /// Timestamp represents UTC time.
    #[deku(id = 0x1)]
    Utc,
    /// Timestamp represents GPS time.
    #[deku(id = 0x2)]
    Gps,
    /// Timestamp represents some other time.
    #[deku(id = 0x3)]
    Other,
}

impl TryFrom<u8> for Tsi {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Tsi::Null as u8 => Ok(Tsi::Null),
            x if x == Tsi::Utc as u8 => Ok(Tsi::Utc),
            x if x == Tsi::Gps as u8 => Ok(Tsi::Gps),
            x if x == Tsi::Other as u8 => Ok(Tsi::Other),
            _ => Err(()),
        }
    }
}

/// TimeStamp-Fractional (TSF) field.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8", endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tsf {
    /// Timestamp is not included.
    #[deku(id = 0x0)]
    Null,
    /// Timestamp represents a sample counter.
    #[deku(id = 0x1)]
    SampleCount,
    /// Timestamp represents a real fractional time (in picoseconds).
    #[deku(id = 0x2)]
    RealTimePs,
    /// Timestamp represents a free-running count.
    #[deku(id = 0x3)]
    FreeRunningCount,
}

impl TryFrom<u8> for Tsf {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == Tsf::Null as u8 => Ok(Tsf::Null),
            x if x == Tsf::SampleCount as u8 => Ok(Tsf::SampleCount),
            x if x == Tsf::RealTimePs as u8 => Ok(Tsf::RealTimePs),
            x if x == Tsf::FreeRunningCount as u8 => Ok(Tsf::FreeRunningCount),
            _ => Err(()),
        }
    }
}

impl PacketHeader {
    /// Gets the raw 32-bit value of the packet header.
    pub fn as_u32(&self) -> u32 {
        ((self.hword_1 as u32) << 16) | ((self.packet_size as u32) & 0xFFFF)
    }
    /// Gets the packet type.
    pub fn packet_type(&self) -> PacketType {
        (((self.hword_1 >> 12) & 0b1111) as u8).try_into().unwrap()
    }
    /// Sets the packet type.
    pub fn set_packet_type(&mut self, packet_type: PacketType) {
        self.hword_1 &= !(0b1111 << 12);
        self.hword_1 |= (packet_type as u16) << 12
    }
    /// Returns true if a class identifier is included in the packet.
    pub fn class_id_included(&self) -> bool {
        self.hword_1 & (1 << 11) > 0
    }

    /// Sets the class_id_included flag.
    pub(crate) fn set_class_id_included(&mut self, included: bool) {
        self.hword_1 = (self.hword_1 & !(1 << 11)) | ((included as u16) << 11);
    }

    /// Returns the packet indicators.
    /// Note: these indicators will be different depending on
    /// the type of packet you're working with, so you'll need
    /// to disambiguate.
    ///
    /// # Example
    /// ```
    /// # use vita49::prelude::*;
    /// use vita49::Indicators;
    /// # fn main() -> Result<(), VitaError> {
    /// # let mut packet = Vrt::new_signal_data_packet();
    /// match packet.header().indicators() {
    ///     Indicators::SignalData(i) => {
    ///         println!("Trailer included: {}", i.trailer_included);
    ///     },
    ///     _ => panic!("unexpected indicators")
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn indicators(&self) -> Indicators {
        Indicators::from_u16_for_packet(self.hword_1, self.packet_type())
    }
    /// Sets the header indicators.
    pub fn set_indicators(&mut self, indicators: Indicators) {
        let flags = indicators.as_u16() & Indicators::MASK;
        self.hword_1 = (self.hword_1 & !Indicators::MASK) | flags;
    }
    /// Returns Ok(true) if the packet is an Ack packet, Ok(false) if
    /// it's some other kind of Command packet, and an error if it's
    /// some other type of packet entirely.
    pub fn is_ack_packet(&self) -> Result<bool, VitaError> {
        match self.indicators() {
            Indicators::Command(i) => Ok(i.ack_packet),
            _ => Err(VitaError::CommandOnly),
        }
    }
    /// Returns Ok(true) if the packet is an Ack packet, Ok(false) if
    /// it's some other kind of Command packet, and an error if it's
    /// some other type of packet entirely.
    pub fn is_cancellation_packet(&self) -> Result<bool, VitaError> {
        match self.indicators() {
            Indicators::Command(i) => Ok(i.cancellation_packet),
            _ => Err(VitaError::CommandOnly),
        }
    }
    /// Gets the TimeStamp-Integer (TSI) field.
    pub fn tsi(&self) -> Tsi {
        (((self.hword_1 >> 6) & 0b11) as u8).try_into().unwrap()
    }

    /// Sets the TimeStamp-Integer (TSI) field.
    pub(crate) fn set_tsi(&mut self, tsi: Tsi) {
        self.hword_1 = (self.hword_1 & !(0b11 << 6)) | ((tsi as u16) << 6);
    }

    /// Gets the TimeStamp-Fractional (TSF) field.
    pub fn tsf(&self) -> Tsf {
        (((self.hword_1 >> 4) & 0b11) as u8).try_into().unwrap()
    }

    /// Sets the TimeStamp-Fractional (TSF) field.
    pub(crate) fn set_tsf(&mut self, tsf: Tsf) {
        self.hword_1 = (self.hword_1 & !(0b11 << 4)) | ((tsf as u16) << 4);
    }

    /// Gets the modulo-16 packet counter field.
    pub fn packet_count(&self) -> u8 {
        (self.hword_1 & 0b1111) as u8
    }
    /// Sets the modulo-16 packet counter field.
    pub fn set_packet_count(&mut self, count: u8) {
        let masked_count = (count & 0b1111) as u16;
        self.hword_1 = (self.hword_1 & (!0b1111)) | masked_count;
    }
    /// Increments the packet counter by one (wrapping at 16).
    pub fn inc_packet_count(&mut self) {
        self.set_packet_count((self.packet_count() + 1) % 16);
    }

    /// Gets the packet size field (32-bit words).
    pub fn packet_size(&self) -> u16 {
        self.packet_size
    }
    /// Sets the packet size field (32-bit words).
    pub fn set_packet_size(&mut self, n_words: u16) {
        self.packet_size = n_words;
    }

    /// Returns true if a stream ID is included, false if not.
    pub fn stream_id_included(&self) -> bool {
        !matches!(
            &self.packet_type(),
            PacketType::SignalDataWithoutStreamId | PacketType::ExtensionDataWithoutStreamId
        )
    }

    /// Returns true if an integer timestamp is included, false if not.
    pub fn integer_timestamp_included(&self) -> bool {
        self.tsi() != Tsi::Null
    }

    /// Returns true if a fractional timestamp is included, false if not.
    pub fn fractional_timestamp_included(&self) -> bool {
        self.tsf() != Tsf::Null
    }

    /// Returns true if a trailer is included, false if not.
    pub fn trailer_included(&self) -> bool {
        match &self.indicators() {
            Indicators::SignalData(i) => i.trailer_included,
            _ => false,
        }
    }

    /// Returns the payload size in 32-bit words.
    pub fn payload_size_words(&self) -> usize {
        // Start with packet size minus 32 bits for the packet header
        let mut ret = self.packet_size as usize - 1;
        if self.stream_id_included() {
            ret -= 1;
        }
        if self.class_id_included() {
            ret -= 2;
        }
        if self.integer_timestamp_included() {
            ret -= 1;
        }
        if self.fractional_timestamp_included() {
            ret -= 2;
        }
        if self.trailer_included() {
            ret -= 1;
        }
        ret
    }

    /// Creates a new signal data packet header with some sane defaults.
    pub fn new_signal_data_header() -> PacketHeader {
        let mut ret = PacketHeader {
            hword_1: 0,
            packet_size: 0,
        };
        ret.set_packet_type(PacketType::SignalData);
        ret.set_indicators(Indicators::SignalData(SignalDataIndicators {
            trailer_included: false,
            not_a_vita490_packet: false,
            signal_spectral_data: false,
        }));
        ret
    }

    /// Creates a new context packet header with some sane defaults.
    pub fn new_context_header() -> PacketHeader {
        let mut ret = PacketHeader {
            hword_1: 0,
            packet_size: 0,
        };
        ret.set_packet_type(PacketType::Context);
        ret.set_indicators(Indicators::Context(ContextIndicators {
            not_a_vita490_packet: false,
            timestamp_mode: TimestampMode::GeneralTiming,
        }));
        ret
    }

    /// Creates a new control packet header.
    pub fn new_control_header() -> PacketHeader {
        let mut ret = PacketHeader::default();
        ret.set_packet_type(PacketType::Command);
        ret.set_indicators(Indicators::Command(CommandIndicators {
            ack_packet: false,
            cancellation_packet: false,
        }));
        ret
    }

    /// Creates a new cancellation packet header.
    pub fn new_cancellation_header() -> PacketHeader {
        let mut ret = PacketHeader::default();
        ret.set_packet_type(PacketType::Command);
        ret.set_indicators(Indicators::Command(CommandIndicators {
            ack_packet: false,
            cancellation_packet: true,
        }));
        ret
    }

    /// Creates a new ack packet header.
    pub fn new_ack_header() -> PacketHeader {
        let mut ret = PacketHeader::default();
        ret.set_packet_type(PacketType::Command);
        ret.set_indicators(Indicators::Command(CommandIndicators {
            ack_packet: true,
            cancellation_packet: false,
        }));
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::Indicators;

    #[test]
    fn packet_header() {
        use crate::prelude::*;
        let packet = Vrt::new_control_packet();
        assert_eq!(packet.header().packet_type(), PacketType::Command);
        assert_eq!(packet.header().as_u32() >> 28, 0b0110);
    }

    #[test]
    fn set_class_id_sets_class_id_included_bit() {
        use crate::prelude::*;
        // Create a new packet (maybe SignalData or Context packet depending on your use case)
        let mut packet = Vrt::new_signal_data_packet();

        // Initially the class_id_included bit should be false
        assert!(!packet.header().class_id_included());

        // Set the class_id
        let class_id = Some(ClassIdentifier::default());
        packet.set_class_id(class_id);

        // Now the class_id_included bit should be true
        assert!(packet.header().class_id_included());
    }

    #[test]
    fn can_toggle_indicators() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();

        // Initially, trailer_included should be false
        assert!(
            matches!(packet.header().indicators(), Indicators::SignalData(si) if !si.trailer_included)
        );

        // Turn on trailer_included
        packet
            .header_mut()
            .set_indicators(Indicators::SignalData(SignalDataIndicators {
                trailer_included: true,
                ..Default::default()
            }));

        // Now, trailer_included should be true
        assert!(
            matches!(packet.header().indicators(), Indicators::SignalData(si) if si.trailer_included)
        );

        // Turn trailer_included back off...
        packet
            .header_mut()
            .set_indicators(Indicators::SignalData(SignalDataIndicators {
                trailer_included: false,
                ..Default::default()
            }));

        // Now, trailer_included should be false again
        assert!(
            matches!(packet.header().indicators(), Indicators::SignalData(si) if !si.trailer_included)
        );
    }

    macro_rules! assert_indicators {
        ($bits:expr, $ptype:expr, $variant:ident { $($field:ident: $val:expr),+ $(,)? }) => {{
            let i = Indicators::from_u16_for_packet($bits << Indicators::OFFSET, $ptype);
            assert!(
                matches!(i, Indicators::$variant(si) if $( si.$field == $val )&&+),
                "indicator mismatch for bits={:#05b}: {:?}",
                $bits,
                i,
            );
        }};
    }

    #[test]
    fn sets_signal_indicator_bits() {
        use crate::prelude::*;
        assert_indicators!(
            0b000,
            PacketType::SignalData,
            SignalData {
                trailer_included: false,
                not_a_vita490_packet: false,
                signal_spectral_data: false
            }
        );
        assert_indicators!(
            0b100,
            PacketType::SignalData,
            SignalData {
                trailer_included: true,
                not_a_vita490_packet: false,
                signal_spectral_data: false
            }
        );
        assert_indicators!(
            0b010,
            PacketType::SignalData,
            SignalData {
                trailer_included: false,
                not_a_vita490_packet: true,
                signal_spectral_data: false
            }
        );
        assert_indicators!(
            0b001,
            PacketType::SignalData,
            SignalData {
                trailer_included: false,
                not_a_vita490_packet: false,
                signal_spectral_data: true
            }
        );
        assert_indicators!(
            0b111,
            PacketType::SignalData,
            SignalData {
                trailer_included: true,
                not_a_vita490_packet: true,
                signal_spectral_data: true
            }
        );
    }

    #[test]
    fn sets_context_indicator_bits() {
        use crate::prelude::*;
        assert_indicators!(
            0b000,
            PacketType::Context,
            Context {
                not_a_vita490_packet: false,
                timestamp_mode: TimestampMode::PreciseTiming
            }
        );
        assert_indicators!(
            0b010,
            PacketType::Context,
            Context {
                not_a_vita490_packet: true,
                timestamp_mode: TimestampMode::PreciseTiming
            }
        );
        assert_indicators!(
            0b001,
            PacketType::Context,
            Context {
                not_a_vita490_packet: false,
                timestamp_mode: TimestampMode::GeneralTiming
            }
        );
        assert_indicators!(
            0b011,
            PacketType::Context,
            Context {
                not_a_vita490_packet: true,
                timestamp_mode: TimestampMode::GeneralTiming
            }
        );
    }

    #[test]
    fn sets_command_indicator_bits() {
        use crate::prelude::*;
        assert_indicators!(
            0b000,
            PacketType::Command,
            Command {
                ack_packet: false,
                cancellation_packet: false
            }
        );
        assert_indicators!(
            0b100,
            PacketType::Command,
            Command {
                ack_packet: true,
                cancellation_packet: false
            }
        );
        assert_indicators!(
            0b001,
            PacketType::Command,
            Command {
                ack_packet: false,
                cancellation_packet: true
            }
        );
        assert_indicators!(
            0b101,
            PacketType::Command,
            Command {
                ack_packet: true,
                cancellation_packet: true
            }
        );
    }
}
