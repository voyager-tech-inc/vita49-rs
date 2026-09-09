// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use deku::prelude::*;
use deku::writer::Writer;
use std::io::{Seek, Write};

use crate::packet_header::PacketHeader;
use crate::payload::Payload;

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
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let sig_data = packet.payload_mut().signal_data_mut()?;
    /// sig_data.set_payload(&[1, 2, 3, 4, 5, 6, 7, 8]);
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_payload(&mut self, bytes: impl Into<Vec<u8>>) {
        self.data = bytes.into()
    }

    /// Gets the size of the payload in 32-bit words.
    pub fn size_words(&self) -> u16 {
        // Ceiling division to make sure we account for padding
        ((self.data.len() + 3) / 4) as u16
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
        let mut final_data = std::borrow::Cow::Borrowed(data);

        if endian == deku::ctx::Endian::Little {
            let mut swapped = data.to_vec();
            for chunk in swapped.chunks_exact_mut(4) {
                chunk.reverse();
            }
            final_data = std::borrow::Cow::Owned(swapped);
        }

        writer.write_bytes(final_data.as_ref())?;

        // Handle zero-padding to match 32-bit words
        let remainder = data.len() % 4;
        if remainder != 0 {
            let pad_len = 4 - remainder;
            let padding = vec![0u8; pad_len];
            writer.write_bytes(&padding)?;
        }

        Ok(())
    }
}
