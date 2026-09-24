// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use deku::prelude::*;
use deku::writer::Writer;
use std::io::{Seek, Write};

use crate::packet_header::PacketHeader;
use crate::payload::Payload;
use crate::VitaError;

/// The VITA 49 packet size field is 16 bits wide and counts 32-bit words, so
/// no payload can be longer than this many words (just under 256 KiB).
const MAX_PAYLOAD_WORDS: u16 = u16::MAX;

/// Base signal data structure.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, _packet_header: &PacketHeader"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignalData {
    #[deku(
        reader = "Self::read_payload(deku::reader, _packet_header.payload_size_words(), endian)",
        writer = "Self::write_payload(deku::writer, &self.data, endian)"
    )]
    data: Vec<u8>,
}

impl TryFrom<Payload> for SignalData {
    type Error = Payload;

    fn try_from(value: Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::SignalData(c) => Ok(c),
            a => Err(a),
        }
    }
}

impl SignalData {
    /// Create a new, empty signal data packet.
    pub fn new() -> SignalData {
        SignalData::default()
    }

    /// Create a new signal data packet directly from an owned vector (zero-copy).
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let my_data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    /// *packet.payload_mut() = Payload::SignalData(SignalData::from_owned(my_data));
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_owned(data: Vec<u8>) -> SignalData {
        SignalData { data }
    }

    /// Create a new signal data packet from an input slice of bytes.
    /// This allocates a new vector under the hood.
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// *packet.payload_mut() = Payload::SignalData(SignalData::from_bytes(&[1, 2, 3, 4, 5, 6, 7, 8]));
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> SignalData {
        SignalData {
            data: bytes.to_vec(),
        }
    }

    /// Get the data payload as a read-only slice (zero-copy).
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// *packet.payload_mut() = Payload::SignalData(SignalData::from_bytes(&[1, 2, 3, 4, 5, 6, 7, 8]));
    /// let signal_data_payload = packet.payload().signal_data()?;
    /// assert_eq!(signal_data_payload.payload(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[u8] {
        &self.data
    }

    /// Get the data payload as a mutable slice (zero-copy).
    ///
    /// Use this to modify a payload in-place, avoiding the allocation when calling
    /// [`Self::set_payload`]. The length does not change, so there is no need to call
    /// [`crate::Vrt::update_packet_size`]. To change the length and mutate, use
    /// [`Self::resize_payload`].
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4])?;
    /// let sig_data = packet.payload_mut().signal_data_mut()?;
    /// sig_data.payload_mut().reverse();
    /// assert_eq!(packet.signal_payload()?, &[4, 3, 2, 1]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consume the struct and take ownership of the underlying payload bytes (zero-copy).
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    /// let signal_data_payload = packet.into_payload().into_signal_data()?;
    /// let payload_vec = signal_data_payload.into_payload();
    /// assert_eq!(payload_vec, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_payload(self) -> Vec<u8> {
        self.data
    }

    /// Set the packet payload to some raw bytes.
    /// Accepts either a `Vec<u8>` (zero-copy) or a `&[u8]` slice (allocates).
    ///
    /// # Warning
    /// This setter does not update the packet's size field. Prefer
    /// [`Vrt::set_signal_payload`](crate::Vrt::set_signal_payload), which keeps
    /// the header in sync and accounts for the prologue and trailer words when
    /// checking that the payload fits.
    ///
    /// # Errors
    /// A payload longer than `u16::MAX` 32-bit words cannot be described by
    /// the 16-bit VITA 49 packet size field, so it returns
    /// [`VitaError::PacketTooLarge`] with the payload's word count.
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let sig_data = packet.payload_mut().signal_data_mut()?;
    /// sig_data.set_payload(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_payload(&mut self, bytes: impl Into<Vec<u8>>) -> Result<(), VitaError> {
        let data = bytes.into();
        let words = Self::size_words_for(data.len());
        if words > MAX_PAYLOAD_WORDS as usize {
            return Err(VitaError::PacketTooLarge { words });
        }
        self.data = data;
        Ok(())
    }

    /// Resize the data payload.
    ///
    /// Use this to resize a payload before modifying it in-place, avoiding the allocation when
    /// calling [`Self::set_payload`]. Growing the payload zeroes the new bytes, and shrinking it
    /// truncates. The length may change, so you must call
    /// [`crate::Vrt::update_packet_size`]. To keep the length and mutate, use
    /// [`Self::payload_mut`].
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4])?;
    /// let sig_data = packet.payload_mut().signal_data_mut()?;
    /// sig_data.resize_payload(8);
    /// sig_data.payload_mut().copy_from_slice(&[5, 6, 7, 8, 9, 10, 11, 12]);
    /// packet.update_packet_size()?;
    /// assert_eq!(packet.signal_payload()?, &[5, 6, 7, 8, 9, 10, 11, 12]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn resize_payload(&mut self, len: usize) {
        self.data.resize(len, 0);
    }

    /// Gets the size of the payload in 32-bit words.
    ///
    /// This can exceed `u16::MAX` for a payload built with [`Self::from_owned`],
    /// [`Self::from_bytes`] or [`Self::resize_payload`], and
    /// [`Vrt::update_packet_size`](crate::Vrt::update_packet_size) then returns
    /// [`VitaError::PacketTooLarge`].
    pub fn size_words(&self) -> usize {
        Self::size_words_for(self.data.len())
    }

    /// Gets the size of a `len`-byte payload in 32-bit words, counting the
    /// padding that rounds it up to a whole word.
    pub(crate) fn size_words_for(len: usize) -> usize {
        // `usize::div_ceil` needs Rust 1.73, past this crate's MSRV.
        len / 4 + usize::from(len % 4 != 0)
    }

    /// Gets the size of the payload in bytes.
    pub fn payload_size_bytes(&self) -> usize {
        self.data.len()
    }

    fn read_payload<R: std::io::Read + std::io::Seek>(
        reader: &mut deku::reader::Reader<R>,
        words: usize,
        endian: deku::ctx::Endian,
    ) -> Result<Vec<u8>, deku::DekuError> {
        let byte_len = words * 4;

        let mut data = vec![0u8; byte_len];

        reader.read_bytes(byte_len, &mut data)?;

        if endian == deku::ctx::Endian::Little {
            for chunk in data.chunks_exact_mut(4) {
                chunk.reverse();
            }
        }

        Ok(data)
    }

    fn write_payload<W: Write + Seek>(
        writer: &mut Writer<W>,
        data: &[u8],
        endian: deku::ctx::Endian,
    ) -> Result<(), deku::DekuError> {
        let remainder = data.len() % 4;
        let pad_len = if remainder != 0 { 4 - remainder } else { 0 };

        if endian == deku::ctx::Endian::Little {
            let mut padded = Vec::with_capacity(data.len() + pad_len);
            padded.extend_from_slice(data);
            padded.resize(data.len() + pad_len, 0u8);
            for chunk in padded.chunks_exact_mut(4) {
                chunk.reverse();
            }
            writer.write_bytes(&padded)?;
        } else {
            writer.write_bytes(data)?;
            if pad_len != 0 {
                let padding = vec![0u8; pad_len];
                writer.write_bytes(&padding)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_at_the_size_limit_is_accepted() {
        let mut sig_data = SignalData::new();
        sig_data
            .set_payload(vec![0u8; MAX_PAYLOAD_WORDS as usize * 4])
            .unwrap();
        assert_eq!(sig_data.size_words(), usize::from(MAX_PAYLOAD_WORDS));
    }

    #[test]
    fn payload_past_the_size_limit_is_rejected() {
        let mut sig_data = SignalData::new();
        // One byte past the limit still rounds up to an extra word.
        let bytes = vec![0u8; MAX_PAYLOAD_WORDS as usize * 4 + 1];
        assert!(matches!(
            sig_data.set_payload(bytes),
            Err(VitaError::PacketTooLarge { words }) if words == usize::from(MAX_PAYLOAD_WORDS) + 1
        ));
        // The oversized payload must not have been stored.
        assert_eq!(sig_data.payload_size_bytes(), 0);
    }
    use std::io::Cursor;

    #[test]
    fn little_endian_padded_payload_round_trip() {
        // A 6-byte payload requires 2 bytes of padding to form two 32-bit words (8 bytes).
        let raw_payload = vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

        // Write in Little Endian
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(Cursor::new(&mut buf));
            SignalData::write_payload(&mut writer, &raw_payload, deku::ctx::Endian::Little)
                .unwrap();
            writer.finalize().unwrap();
        }
        assert_eq!(buf.len(), 8, "payload must be padded to 8 bytes (2 words)");

        // Word 0 was [0x11, 0x22, 0x33, 0x44] -> in LE wire order: [0x44, 0x33, 0x22, 0x11]
        assert_eq!(&buf[0..4], &[0x44, 0x33, 0x22, 0x11]);
        // Word 1 was [0x55, 0x66, 0x00, 0x00] -> in LE wire order: [0x00, 0x00, 0x66, 0x55]
        assert_eq!(
            &buf[4..8],
            &[0x00, 0x00, 0x66, 0x55],
            "padding must be reversed along with data bytes in LE word"
        );

        // Read back in Little Endian
        let mut cursor = Cursor::new(&buf);
        let mut reader = deku::reader::Reader::new(&mut cursor);
        let read_data =
            SignalData::read_payload(&mut reader, 2, deku::ctx::Endian::Little).unwrap();

        // Data bytes must be restored to their original positions (followed by zero padding)
        assert_eq!(&read_data[0..6], &raw_payload[..]);
        assert_eq!(&read_data[6..8], &[0x00, 0x00]);
    }
}
