// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Structures and methods related to the class identifier
field (ANSI/VITA-49.2-2017 section 5.1.3).
*/
use deku::prelude::*;

/// Base class identifier data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassIdentifier {
    word_1: u32,
    information_class_code: u16,
    packet_class_code: u16,
}

impl ClassIdentifier {
    /// Gets the number of padding bits.
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_class_id(Some(ClassIdentifier::default()));
    /// packet.class_id_mut().unwrap().set_pad_bit_count(4);
    /// assert_eq!(packet.class_id().unwrap().pad_bit_count(), 4);
    /// ```
    pub fn pad_bit_count(&self) -> u8 {
        ((self.word_1 >> 27) & 0x1F) as u8
    }
    /// Set the number of padding bits.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_class_id(Some(ClassIdentifier::default()));
    /// packet.class_id_mut().unwrap().set_pad_bit_count(4);
    /// assert_eq!(packet.class_id().unwrap().pad_bit_count(), 4);
    /// ```
    pub fn set_pad_bit_count(&mut self, count: u8) {
        self.word_1 = self.word_1 & !(0x1F << 27) | ((count as u32) << 27);
    }

    /// Gets the Organizational Unique Identifier (OUI).
    pub fn oui(&self) -> u32 {
        self.word_1 & 0xFF_FFFF
    }
    /// Sets the Organizational Unique Identifier (OUI).
    ///
    /// Note: while this API takes a 32-bit integer, only the least
    /// significant 24 bits are used.
    pub fn set_oui(&mut self, oui: u32) {
        self.word_1 = self.word_1 & !(0xFF_FFFF) | (oui & 0xFF_FFFF);
    }

    /// Gets the information class code.
    pub fn information_class_code(&self) -> u16 {
        self.information_class_code
    }
    /// Sets the information class code.
    pub fn set_information_class_code(&mut self, code: u16) {
        self.information_class_code = code;
    }

    /// Gets the packet class code.
    pub fn packet_class_code(&self) -> u16 {
        self.packet_class_code
    }
    /// Sets the packet class code.
    pub fn set_packet_class_code(&mut self, code: u16) {
        self.packet_class_code = code;
    }
}
