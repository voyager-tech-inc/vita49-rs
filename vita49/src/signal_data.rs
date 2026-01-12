// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use deku::prelude::*;

use crate::packet_header::PacketHeader;
use crate::payload::Payload;
use crate::VitaError;

/// Base signal data structure.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, _packet_header: &PacketHeader"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignalData {
    #[deku(count = "_packet_header.payload_size_words()")]
    data: Vec<u32>,
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

    /// Create a new signal data packet from an input slice of bytes.
    ///
    /// # Errors
    /// Internally, the payload is represented as a vector of 32-bit integers.
    /// If you pass a payload of bytes with a length indivisible by 4, the call
    /// will return an error.
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// *packet.payload_mut() = Payload::SignalData(SignalData::from_bytes(&vec![1, 2, 3, 4, 5, 6, 7, 8])?);
    /// assert_eq!(packet.signal_payload()?, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<SignalData, VitaError> {
        let mut ret = SignalData::new();
        ret.set_payload(bytes)?;
        Ok(ret)
    }

    /// Get the data payload as a vector of bytes.
    pub fn payload(&self) -> Vec<u8> {
        self.data.iter().flat_map(|&v| v.to_be_bytes()).collect()
    }

    /// Set the packet payload to some raw bytes.
    ///
    /// # Errors
    /// Internally, the payload is represented as a vector of 32-bit integers.
    /// If you pass a payload of bytes with a length indivisible by 4, the call
    /// will return an error.
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let sig_data = packet.payload_mut().signal_data_mut()?;
    /// sig_data.set_payload(&vec![1, 2, 3, 4, 5, 6, 7, 8])?;
    /// assert_eq!(packet.signal_payload()?, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_payload(&mut self, bytes: &[u8]) -> Result<(), VitaError> {
        let packed_payload: Vec<u32> = bytes
            .chunks(4)
            .map(|chunk| {
                chunk
                    .try_into()
                    .map(u32::from_be_bytes)
                    .map_err(|_| VitaError::PayloadUneven32BitWords)
            })
            .collect::<Result<Vec<u32>, VitaError>>()?;
        self.data = packed_payload.to_vec();
        Ok(())
    }

    /// Gets the size of the payload in 32-bit words.
    pub fn size_words(&self) -> u16 {
        self.data.len() as u16
    }

    /// Gets the size of the payload in bytes.
    pub fn payload_size_bytes(&self) -> usize {
        self.data.len() * 4
    }
}
