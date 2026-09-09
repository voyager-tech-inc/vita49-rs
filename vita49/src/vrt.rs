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
    pub fn header_mut(&mut self) -> &mut PacketHeader {
        &mut self.header
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

    /// Set the packet payload to some raw bytes (signal data only).
    /// Can be an owned `Vec<u8>` (zero-copy) or a `&[u8]` slice which
    /// will allocate under the hood.
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
    /// assert_eq!(packet.signal_payload()?, &[1, 2, 3, 4, 5, 6, 7, 8]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_signal_payload(&mut self, payload: impl Into<Vec<u8>>) -> Result<(), VitaError> {
        let sig_data = self.payload.signal_data_mut()?;
        sig_data.set_payload(payload);
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
        let mut packet_size_words = 1;
        if self.header.stream_id_included() {
            packet_size_words += 1;
        }
        if self.header.class_id_included() {
            packet_size_words += 2;
        }
        if self.header.integer_timestamp_included() {
            packet_size_words += 1;
        }
        if self.header.fractional_timestamp_included() {
            packet_size_words += 2;
        }
        if self.header.trailer_included() {
            packet_size_words += 1;
        }

        packet_size_words += self.payload.size_words();

        self.header.set_packet_size(packet_size_words);
    }
}

#[cfg(test)]
mod tests {

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
}
