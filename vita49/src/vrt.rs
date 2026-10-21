// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Primary module for parsing/generating VRT data. This should
be the main entrypoint for any users of this crate.
*/

use crate::command_prelude::*;
use crate::prelude::*;
use crate::Trailer;
use deku::prelude::*;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(endian = "big")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The main VRT data structure that encapsulates all types
/// of VRT packets.
pub struct Vrt {
    /// VRT packet header (present on all packets).
    #[deku(writer = "Vrt::write_header(deku::writer, header, stream_id, payload)")]
    header: PacketHeader,
    /// Stream identifier.
    #[deku(cond = "header.stream_id_included()")]
    stream_id: Option<u32>,
    /// Class identifier.
    #[deku(cond = "header.class_id_included()")]
    class_id: Option<ClassIdentifier>,
    /// Integer timestamp.
    #[deku(cond = "header.integer_timestamp_included()")]
    integer_timestamp: Option<u32>,
    /// Fractional timestamp.
    #[deku(cond = "header.fractional_timestamp_included()")]
    fractional_timestamp: Option<u64>,
    /// Packet payload. For signal data, this would be raw bytes. For
    /// context, this would be context information, etc..
    #[deku(ctx = "header")]
    payload: Payload,
    /// Data trailer.
    #[deku(cond = "header.trailer_included()")]
    trailer: Option<Trailer>,
}

impl Vrt {
    /// Writes the header after checking that its packet type matches the
    /// payload variant and the stream ID's presence.
    fn write_header<W: std::io::Write + std::io::Seek>(
        writer: &mut deku::writer::Writer<W>,
        header: &PacketHeader,
        stream_id: &Option<u32>,
        payload: &Payload,
    ) -> Result<(), DekuError> {
        // The setters keep the header and the fields in step, but `header_mut` and `payload_mut`
        // can change either side alone, and the bytes would then read back as a different packet.
        // The check runs before the first byte so a refused packet leaves nothing in the writer.
        let packet_type = header.packet_type();
        if !payload.packet_types().contains(&packet_type) {
            let err = VitaError::WrongPacketType {
                expected: payload.packet_types(),
                actual: packet_type,
            };
            return Err(DekuError::InvalidParam(err.to_string().into()));
        }
        if header.stream_id_included() != stream_id.is_some() {
            return Err(DekuError::InvalidParam(
                format!("packet type {packet_type:?} does not match stream ID {stream_id:?}")
                    .into(),
            ));
        }
        header.to_writer(writer, deku::ctx::Endian::Big)
    }

    /// Produce a new signal data packet with some sane defaults.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_stream_id(Some(0xDEADBEEF));
    /// packet.set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    /// assert_eq!(packet.stream_id(), Some(0xDEADBEEF));
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_signal_data_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_signal_data_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::SignalData(SignalData::new()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Produce a new context packet with some sane defaults.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_context_packet();
    /// let context: &mut Context = packet.payload_mut().context_mut().unwrap();
    /// context.set_bandwidth_hz(Some(8e6));
    /// assert_eq!(context.bandwidth_hz(), Some(8e6));
    /// ```
    pub fn new_context_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_context_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::Context(Context::new()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Produce a new control packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_control_packet();
    /// let mut command = packet.payload_mut().command_mut().unwrap();
    /// let mut control = command.payload_mut().control_mut().unwrap();
    /// control.set_bandwidth_hz(Some(8e6));
    /// assert_eq!(control.bandwidth_hz(), Some(8e6));
    /// ```
    pub fn new_control_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_control_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::Command(Command::new_control()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Produce a new cancellation packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_cancellation_packet();
    /// let mut command = packet.payload_mut().command_mut().unwrap();
    /// let mut cancel = command.payload_mut().cancellation_mut().unwrap();
    /// cancel.cif0_mut().set_bandwidth();
    /// assert!(cancel.cif0().bandwidth());
    /// ```
    pub fn new_cancellation_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_cancellation_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::Command(Command::new_cancellation()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Produce a new validation ACK packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let mut packet = Vrt::new_validation_ack_packet();
    /// let mut command = packet.payload_mut().command_mut().unwrap();
    /// let mut ack = command.payload_mut().validation_ack_mut().unwrap();
    /// ack.set_bandwidth(AckLevel::Warning, Some(AckResponse::default()));
    /// assert!(ack.bandwidth().is_some());
    /// ```
    pub fn new_validation_ack_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_ack_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::Command(Command::new_validation_ack()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Produce a new execution ACK packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let mut packet = Vrt::new_exec_ack_packet();
    /// let mut command = packet.payload_mut().command_mut().unwrap();
    /// let mut ack = command.payload_mut().exec_ack_mut().unwrap();
    /// ack.set_bandwidth(AckLevel::Warning, Some(AckResponse::default()));
    /// assert!(ack.bandwidth().is_some());
    /// ```
    pub fn new_exec_ack_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_ack_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::Command(Command::new_exec_ack()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Produce a new query ACK packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let mut packet = Vrt::new_query_ack_packet();
    /// let mut command = packet.payload_mut().command_mut().unwrap();
    /// let mut ack = command.payload_mut().query_ack_mut().unwrap();
    /// ack.set_bandwidth_hz(Some(100e6));
    /// assert!(ack.bandwidth_hz().is_some());
    /// ```
    pub fn new_query_ack_packet() -> Vrt {
        let mut ret = Vrt {
            header: PacketHeader::new_ack_header(),
            stream_id: Some(0),
            class_id: None,
            integer_timestamp: None,
            fractional_timestamp: None,
            payload: Payload::Command(Command::new_query_ack()),
            trailer: None,
        };
        ret.update_packet_size();
        ret
    }

    /// Gets a reference to the packet header.
    pub fn header(&self) -> &PacketHeader {
        &self.header
    }
    /// Gets a mutable reference to the packet header.
    ///
    /// Use [`Self::set_packet_type`] to change the packet type. Setting it
    /// through [`PacketHeader::set_packet_type`] does not check it against
    /// the payload, and serializing a packet whose header and payload
    /// disagree returns an error.
    pub fn header_mut(&mut self) -> &mut PacketHeader {
        &mut self.header
    }

    /// Sets the packet type.
    ///
    /// The payload variant decides which packet types a packet can take, as
    /// [`Payload::packet_types`] lists. For example, a signal data packet can
    /// become an extension data packet, but not a context packet.
    ///
    /// A type without a stream ID clears the stream ID, and a type with one
    /// sets a missing stream ID to 0. The packet size is updated
    /// automatically.
    ///
    /// # Errors
    /// A packet type the payload does not accept returns
    /// [`VitaError::WrongPacketType`] and leaves the packet unchanged.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_packet_type(PacketType::ExtensionData)?;
    /// assert_eq!(packet.header().packet_type(), PacketType::ExtensionData);
    ///
    /// assert!(matches!(
    ///     packet.set_packet_type(PacketType::Context),
    ///     Err(VitaError::WrongPacketType { .. })
    /// ));
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_packet_type(&mut self, packet_type: PacketType) -> Result<(), VitaError> {
        let expected = self.payload.packet_types();
        if !expected.contains(&packet_type) {
            return Err(VitaError::WrongPacketType {
                expected,
                actual: packet_type,
            });
        }
        self.header.set_packet_type(packet_type);
        if !self.header.stream_id_included() {
            self.stream_id = None;
        } else if self.stream_id.is_none() {
            self.stream_id = Some(0);
        }
        self.update_packet_size();
        Ok(())
    }

    /// Get the packet stream ID.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_stream_id(Some(0xDEADBEEF));
    /// assert_eq!(packet.stream_id(), Some(0xDEADBEEF));
    /// ```
    pub fn stream_id(&self) -> Option<u32> {
        self.stream_id
    }

    /// Sets the packet's stream ID. If `None` is passed, the stream ID
    /// field will be unset.
    ///
    /// Note: if the packet type does not match after setting/unsetting,
    /// the packet type will be updated to reflect the change. For example,
    /// if you did `packet.set_stream_id(1)` on a `PacketType::SignalDataWithoutStreamId`,
    /// it would change the packet to a `PacketType:SignalData`.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_stream_id(Some(0xDEADBEEF));
    /// assert_eq!(packet.stream_id(), Some(0xDEADBEEF));
    /// assert!(matches!(packet.header().packet_type(), PacketType::SignalData));
    /// packet.set_stream_id(None);
    /// assert!(matches!(packet.header().packet_type(), PacketType::SignalDataWithoutStreamId));
    /// ```
    pub fn set_stream_id(&mut self, stream_id: Option<u32>) {
        if stream_id.is_none() && !self.header.packet_type().has_signal_data_payload() {
            // Per ANSI/VITA-49.2 Rule 5.1.2-1, the Stream Identifier shall be present in all
            // Context packets, Extension Context packets, Command packets, and Extension Command packets.
            // Default to Some(0) to avoid omitting mandatory wire bytes and desynchronizing the stream.
            self.stream_id = Some(0);
            return;
        }
        self.stream_id = stream_id;
        if self.stream_id.is_some() {
            match self.header.packet_type() {
                PacketType::SignalDataWithoutStreamId => {
                    self.header.set_packet_type(PacketType::SignalData);
                }
                PacketType::ExtensionDataWithoutStreamId => {
                    self.header.set_packet_type(PacketType::ExtensionData);
                }
                _ => (),
            }
        } else {
            match self.header.packet_type() {
                PacketType::SignalData => {
                    self.header
                        .set_packet_type(PacketType::SignalDataWithoutStreamId);
                }
                PacketType::ExtensionData => {
                    self.header
                        .set_packet_type(PacketType::ExtensionDataWithoutStreamId);
                }
                _ => (),
            }
        }
    }

    /// Gets a reference to the packet class identifier.
    pub fn class_id(&self) -> Option<&ClassIdentifier> {
        self.class_id.as_ref()
    }
    /// Gets the packet class identifier as a mutable reference.
    pub fn class_id_mut(&mut self) -> Option<&mut ClassIdentifier> {
        self.class_id.as_mut()
    }
    /// Set the packet class identifier.
    pub fn set_class_id(&mut self, class_id: Option<ClassIdentifier>) {
        self.class_id = class_id;
        self.header.set_class_id_included(class_id.is_some());
    }

    /// Gets the integer timestamp field.
    pub fn integer_timestamp(&self) -> Option<u32> {
        self.integer_timestamp
    }
    /// Sets the integer timestamp field.
    ///
    /// When setting this field, you must also provide a [`Tsi`] mode to indicate what
    /// kind of timestamp is being represented.
    ///
    /// # Errors
    /// If a timestamp and tsi mode are passed that don't work together, this function
    /// will return an error. For example, if `timestamp = Some(123)` and `tsi = Tsi::Null`.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_integer_timestamp(Some(12345), Tsi::Utc)?;
    /// assert_eq!(packet.integer_timestamp(), Some(12345));
    /// # Ok(())
    /// # }
    /// ```
    /// ```should_panic
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// // This call will return an error
    /// packet.set_integer_timestamp(Some(12345), Tsi::Null)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_integer_timestamp(
        &mut self,
        timestamp: Option<u32>,
        tsi: Tsi,
    ) -> Result<(), VitaError> {
        if (timestamp.is_some() && matches!(tsi, Tsi::Null))
            || (timestamp.is_none() && !matches!(tsi, Tsi::Null))
        {
            return Err(VitaError::TimestampModeMismatch);
        }
        self.integer_timestamp = timestamp;
        self.header.set_tsi(tsi);
        Ok(())
    }

    /// Gets the fractional timestamp field.
    pub fn fractional_timestamp(&self) -> Option<u64> {
        self.fractional_timestamp
    }
    /// Sets the fractional timestamp field.
    ///
    /// When setting this field, you must also provide a [`Tsf`] mode to indicate what
    /// kind of timestamp is being represented.
    ///
    /// # Errors
    /// If a timestamp and tsi mode are passed that don't work together, this function
    /// will return an error. For example, if `timestamp = Some(123)` and `tsi = Tsi::Null`.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_fractional_timestamp(Some(12345), Tsf::SampleCount)?;
    /// assert_eq!(packet.fractional_timestamp(), Some(12345));
    /// # Ok(())
    /// # }
    /// ```
    /// ```should_panic
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// // This call will return an error
    /// packet.set_fractional_timestamp(Some(12345), Tsf::Null)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_fractional_timestamp(
        &mut self,
        timestamp: Option<u64>,
        tsf: Tsf,
    ) -> Result<(), VitaError> {
        if (timestamp.is_some() && matches!(tsf, Tsf::Null))
            || (timestamp.is_none() && !matches!(tsf, Tsf::Null))
        {
            return Err(VitaError::TimestampModeMismatch);
        }
        self.fractional_timestamp = timestamp;
        self.header.set_tsf(tsf);
        Ok(())
    }

    /// Gets a reference to the payload enumeration.
    pub fn payload(&self) -> &Payload {
        &self.payload
    }

    /// Consumes the struct and returns the inner payload enumeration.
    pub fn into_payload(self) -> Payload {
        self.payload
    }

    /// Gets a mutable reference to the payload enumeration.
    pub fn payload_mut(&mut self) -> &mut Payload {
        &mut self.payload
    }

    /// Gets a reference to the trailer.
    pub fn trailer(&self) -> Option<&Trailer> {
        self.trailer.as_ref()
    }

    /// Gets a mutable reference to the trailer.
    pub fn trailer_mut(&mut self) -> Option<&mut Trailer> {
        self.trailer.as_mut()
    }

    /// Adds (or removes) a packet trailer.
    ///
    /// # Example
    /// ```
    /// # use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let mut trailer = Trailer::default();
    /// trailer.set_agc_indicator(Some(true));
    /// packet.set_trailer(Some(trailer));
    /// assert!(packet.header().trailer_included());
    /// assert!(packet.trailer().is_some());
    /// assert!(packet.trailer().and_then(|t| t.agc_indicator()).is_some_and(|agc| agc));
    /// packet.set_trailer(None);
    /// assert!(!packet.header().trailer_included());
    /// assert!(packet.trailer().is_none());
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_trailer(&mut self, trailer: Option<Trailer>) -> Result<(), VitaError> {
        match &self.payload {
            Payload::SignalData(_) => {
                self.trailer = trailer;
                self.header.set_trailer_included(self.trailer.is_some());
                Ok(())
            }
            _ => Err(VitaError::SignalDataOnly),
        }
    }

    /// Get a read-only slice of the packet payload.
    ///
    /// # Errors
    /// This function should only be used with a signal data packet type. Use
    /// of this function on other packet types will return an error.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn signal_payload(&self) -> Result<&[u8], VitaError> {
        Ok(self.payload.signal_data()?.payload())
    }

    /// Get a mutable slice of the packet payload.
    ///
    /// Use this to modify a payload in-place, avoiding the allocation when calling
    /// [`Self::set_signal_payload`]. The length does not change, so there is no need to call
    /// [`crate::Vrt::update_packet_size`]. To change the length and mutate, use
    /// [`Self::resize_signal_payload`].
    ///
    /// # Errors
    /// This function should only be used with a signal data packet type. Use
    /// of this function on other packet types will return an error.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4])?;
    /// packet.signal_payload_mut()?.reverse();
    /// assert_eq!(packet.signal_payload()?, &[4, 3, 2, 1]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn signal_payload_mut(&mut self) -> Result<&mut [u8], VitaError> {
        Ok(self.payload.signal_data_mut()?.payload_mut())
    }

    /// Resize the packet payload.
    ///
    /// Use this to resize a payload before modifying it in-place, avoiding the allocation when
    /// calling [`Self::set_signal_payload`]. Growing the payload zeroes the new bytes, and
    /// shrinking it truncates. The length may change and the packet size is updated
    /// automatically so calling [`crate::Vrt::update_packet_size`] is not necessary. To keep the
    /// length and mutate, use [`Self::signal_payload_mut`].
    ///
    /// # Errors
    /// This function should only be used with a signal data packet type. Use
    /// of this function on other packet types will return an error.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4])?;
    /// packet.resize_signal_payload(8)?;
    /// packet.signal_payload_mut()?.copy_from_slice(&[5, 6, 7, 8, 9, 10, 11, 12]);
    /// assert_eq!(packet.signal_payload()?, &[5, 6, 7, 8, 9, 10, 11, 12]);
    /// assert_eq!(packet.header().packet_size(), 4);
    /// # Ok(())
    /// # }
    /// ```
    pub fn resize_signal_payload(&mut self, len: usize) -> Result<(), VitaError> {
        self.payload.signal_data_mut()?.resize_payload(len);
        self.update_packet_size();
        Ok(())
    }

    /// Set the packet payload to some raw bytes (signal data only).
    /// Can be an owned `Vec<u8>` (zero-copy) or a `&[u8]` slice which
    /// will allocate under the hood.
    ///
    /// # Errors
    /// This function should only be used with a signal data packet type. Use
    /// of this function on other packet types will return an error.
    ///
    /// An error is also returned when the payload would not fit alongside the
    /// packet prologue and trailer in the 16-bit VITA 49 packet size field.
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_signal_payload(&mut self, payload: impl Into<Vec<u8>>) -> Result<(), VitaError> {
        let payload = payload.into();
        // The packet size field covers the prologue and trailer as well as the
        // payload, so how much payload fits depends on which optional words are
        // present.
        let room_words = u16::MAX - self.header.min_packet_size_words();
        if payload.len() > room_words as usize * 4 {
            return Err(VitaError::OutOfRange);
        }
        let sig_data = self.payload.signal_data_mut()?;
        sig_data.set_payload(payload)?;
        self.update_packet_size();
        Ok(())
    }

    /// Consume the VRT packet and extract the owned signal data payload.
    /// This avoids cloning the internal vector.
    ///
    /// # Errors
    /// This function should only be used with a signal data packet type. Use
    /// of this function on other packet types will return an error.
    ///
    /// # Example
    /// ```
    /// # use std::io;
    /// use vita49::prelude::*;
    /// # fn main() -> Result<(), VitaError> {
    /// let mut packet = Vrt::new_signal_data_packet();
    /// packet.set_signal_payload(&[1, 2, 3, 4, 5, 6, 7, 8])?;
    /// let payload = packet.into_signal_payload()?;
    /// assert_eq!(payload, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_signal_payload(self) -> Result<Vec<u8>, VitaError> {
        match self.payload {
            Payload::SignalData(sig) => Ok(sig.into_payload()),
            _ => Err(VitaError::SignalDataOnly),
        }
    }

    /// Update the VRT packet header size field to reflect the current contents of
    /// the data structure.
    ///
    /// This function should be executed after making any changes to a packet (i.e
    /// after any functions `set_*()`) to make sure the header size is set correctly
    /// prior to serialization.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_context_packet();
    /// let context = packet.payload_mut().context_mut().unwrap();
    /// context.set_bandwidth_hz(Some(8e6));
    /// context.set_sample_rate_sps(Some(8e6));
    /// packet.update_packet_size();
    /// // ... write the packet
    /// ```
    pub fn update_packet_size(&mut self) {
        let packet_size_words = self.header.min_packet_size_words() + self.payload.size_words();
        self.header.set_packet_size(packet_size_words);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn oversized_signal_payload_is_rejected_rather_than_wrapping() {
        use crate::prelude::*;

        let mut packet = Vrt::new_signal_data_packet();
        let room_words = (u16::MAX - packet.header().min_packet_size_words()) as usize;

        assert!(matches!(
            packet.set_signal_payload(vec![0u8; room_words * 4 + 1]),
            Err(VitaError::OutOfRange)
        ));

        // One word under the ceiling must still be accepted, and the size
        // field must hold the true total rather than a wrapped value.
        packet
            .set_signal_payload(vec![0u8; room_words * 4])
            .unwrap();
        assert_eq!(packet.header().packet_size(), u16::MAX);
    }

    #[test]
    fn packet_type_within_the_payload_family_round_trips() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();
        packet.set_packet_type(PacketType::ExtensionData).unwrap();
        packet.set_signal_payload([1, 2, 3, 4]).unwrap();

        let bytes = packet.to_bytes().unwrap();
        let parsed = Vrt::try_from(&bytes[..]).unwrap();
        assert_eq!(parsed, packet);
        assert_eq!(parsed.signal_payload().unwrap(), &[1, 2, 3, 4]);
    }

    #[test]
    fn packet_type_outside_the_payload_family_is_rejected() {
        use crate::prelude::*;
        let cases = [
            (Vrt::new_signal_data_packet(), PacketType::Context),
            (Vrt::new_signal_data_packet(), PacketType::ExtensionCommand),
            (Vrt::new_context_packet(), PacketType::SignalData),
            (Vrt::new_context_packet(), PacketType::Command),
            (Vrt::new_control_packet(), PacketType::ExtensionData),
            (Vrt::new_control_packet(), PacketType::ExtensionContext),
        ];
        for (mut packet, packet_type) in cases {
            let before = packet.clone();
            assert!(matches!(
                packet.set_packet_type(packet_type),
                Err(VitaError::WrongPacketType { expected, actual })
                    if actual == packet_type && expected == before.payload().packet_types()
            ));
            assert_eq!(packet, before);
        }
    }

    #[test]
    fn packet_type_keeps_the_stream_id_in_step() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();
        packet.set_stream_id(Some(7));
        let with_stream_id = packet.header().packet_size();

        packet
            .set_packet_type(PacketType::ExtensionDataWithoutStreamId)
            .unwrap();
        assert_eq!(packet.stream_id(), None);
        assert_eq!(packet.header().packet_size(), with_stream_id - 1);

        packet.set_packet_type(PacketType::SignalData).unwrap();
        assert_eq!(packet.stream_id(), Some(0));
        assert_eq!(packet.header().packet_size(), with_stream_id);

        let bytes = packet.to_bytes().unwrap();
        assert_eq!(Vrt::try_from(&bytes[..]).unwrap(), packet);
    }

    #[test]
    fn header_type_that_disagrees_with_the_payload_is_not_written() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();
        packet.header_mut().set_packet_type(PacketType::Context);
        let err = packet.to_bytes().unwrap_err();
        assert!(matches!(err, deku::DekuError::InvalidParam(_)));
        assert!(err.to_string().contains("found Context"), "{err}");

        let mut packet = Vrt::new_signal_data_packet();
        *packet.payload_mut() = Payload::Context(Context::new());
        let err = packet.to_bytes().unwrap_err();
        assert!(err.to_string().contains("found SignalData"), "{err}");
    }

    #[test]
    fn every_packet_type_parses_to_a_payload_that_accepts_it() {
        use crate::prelude::*;
        let cases = [
            (
                Vrt::new_signal_data_packet(),
                PacketType::SignalDataWithoutStreamId,
            ),
            (Vrt::new_signal_data_packet(), PacketType::SignalData),
            (
                Vrt::new_signal_data_packet(),
                PacketType::ExtensionDataWithoutStreamId,
            ),
            (Vrt::new_signal_data_packet(), PacketType::ExtensionData),
            (Vrt::new_context_packet(), PacketType::Context),
            (Vrt::new_context_packet(), PacketType::ExtensionContext),
            (Vrt::new_control_packet(), PacketType::Command),
            (Vrt::new_control_packet(), PacketType::ExtensionCommand),
        ];
        for (mut packet, packet_type) in cases {
            packet.set_packet_type(packet_type).unwrap();
            let bytes = packet.to_bytes().unwrap();
            let parsed = Vrt::try_from(&bytes[..]).unwrap();
            assert_eq!(parsed.header().packet_type(), packet_type);
            // The variant the parser picked from the wire type has to accept that type, or
            // the packet could not be written back.
            assert!(parsed.payload().packet_types().contains(&packet_type));
            assert_eq!(parsed, packet);
        }
    }

    #[test]
    fn refused_packet_leaves_nothing_in_the_writer() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();
        packet.set_stream_id(Some(0xAABB_CCDD));
        packet.header_mut().set_packet_type(PacketType::Context);

        let mut out = Vec::new();
        let mut writer = deku::writer::Writer::new(std::io::Cursor::new(&mut out));
        assert!(packet.to_writer(&mut writer, ()).is_err());
        writer.finalize().unwrap();
        assert!(out.is_empty(), "{out:02x?}");
    }

    #[test]
    fn header_type_that_disagrees_with_the_stream_id_is_not_written() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();
        packet
            .header_mut()
            .set_packet_type(PacketType::SignalDataWithoutStreamId);
        let err = packet.to_bytes().unwrap_err();
        assert!(matches!(err, deku::DekuError::InvalidParam(_)));
        assert!(err.to_string().contains("stream ID"), "{err}");
    }

    #[test]
    fn trailer_header_bit_toggle() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();

        packet.set_trailer(Some(Trailer::default())).unwrap();
        assert!(packet.header().trailer_included());

        packet.set_trailer(None).unwrap();
        assert!(!packet.header().trailer_included());
    }

    #[test]
    fn set_trailer_errors_on_non_signal_data() {
        use crate::prelude::*;
        let mut packet = Vrt::new_context_packet();
        assert!(matches!(
            packet.set_trailer(Some(Trailer::default())),
            Err(VitaError::SignalDataOnly)
        ));
    }

    #[test]
    fn context_packet_stream_id_none_defaults_to_some_zero() {
        use crate::prelude::*;
        let mut packet = Vrt::new_context_packet();
        packet.set_stream_id(None);
        // Rule 5.1.2-1: Context packets must include Stream ID
        assert_eq!(packet.stream_id(), Some(0));
        packet.update_packet_size();

        // Must serialize and deserialize cleanly without wire stream corruption
        let bytes = packet.to_bytes().unwrap();
        let parsed = Vrt::try_from(&bytes[..]).unwrap();
        assert_eq!(parsed.stream_id(), Some(0));
        assert_eq!(parsed.header().packet_type(), PacketType::Context);
    }

    #[test]
    fn resize_signal_payload_updates_the_packet_size() {
        use crate::prelude::*;
        let mut packet = Vrt::new_signal_data_packet();
        packet.set_signal_payload([1, 2, 3, 4]).unwrap();
        let before = packet.header().packet_size();

        packet.resize_signal_payload(16).unwrap();
        packet
            .signal_payload_mut()
            .unwrap()
            .copy_from_slice(&[9u8; 16]);

        assert_eq!(packet.signal_payload().unwrap(), &[9u8; 16]);
        assert_eq!(packet.header().packet_size(), before + 3);

        // The size the header states has to survive a round trip through the wire.
        let bytes = packet.to_bytes().unwrap();
        let parsed = Vrt::try_from(&bytes[..]).unwrap();
        assert_eq!(parsed.signal_payload().unwrap(), &[9u8; 16]);
    }

    #[test]
    fn payload_accessors_error_on_non_signal_data() {
        use crate::prelude::*;
        let mut packet = Vrt::new_context_packet();
        assert!(matches!(
            packet.signal_payload_mut(),
            Err(VitaError::SignalDataOnly)
        ));
        assert!(matches!(
            packet.resize_signal_payload(4),
            Err(VitaError::SignalDataOnly)
        ));
    }
}
