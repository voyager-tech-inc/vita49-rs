// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
use deku::prelude::*;

use crate::packet_header::{PacketHeader, PacketType};
use crate::signal_data::SignalData;
use crate::Command;
use crate::Context;
use crate::VitaError;

/// Generic payload enumeration. The payload format will differ depending on the
/// type of packet.
///
/// Normally, when using this enum, you'd unwrap the inner type using one of the
/// helper functions.
///
/// # Example
/// ```
/// use vita49::prelude::*;
/// let mut packet = Vrt::new_context_packet();
/// // Safe to unwrap as you just made it a context packet above.
/// let context = packet.payload_mut().context_mut().unwrap();
/// context.set_bandwidth_hz(Some(8e6));
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, packet_header: &PacketHeader",
    id = "packet_header.packet_type()"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::large_enum_variant)]
pub enum Payload {
    /// Payload for a context packet.
    #[deku(id = "PacketType::Context | PacketType::ExtensionContext")]
    Context(Context),
    /// Payload for a command packet.
    #[deku(id = "PacketType::Command | PacketType::ExtensionCommand")]
    Command(#[deku(ctx = "packet_header")] Command),
    /// Payload for signal data.
    #[deku(id_pat = "_")]
    SignalData(#[deku(ctx = "packet_header")] SignalData),
}

impl Payload {
    /// Gets a reference to the signal data payload. This "unwraps"
    /// the generic `Payload` into a `SignalData` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a signal data packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let signal_data: &SignalData = packet.payload().signal_data().unwrap();
    /// assert_eq!(signal_data.payload_size_bytes(), 0);
    /// ```
    pub fn signal_data(&self) -> Result<&SignalData, VitaError> {
        match self {
            Payload::SignalData(p) => Ok(p),
            _ => Err(VitaError::SignalDataOnly),
        }
    }

    /// Consumes the `Payload` struct and returns the inner `SignalData`
    /// struct.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a signal data packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let signal_data: SignalData = packet.into_payload().into_signal_data().unwrap();
    /// assert_eq!(signal_data.payload_size_bytes(), 0);
    /// ```
    pub fn into_signal_data(self) -> Result<SignalData, VitaError> {
        match self {
            Payload::SignalData(p) => Ok(p),
            _ => Err(VitaError::SignalDataOnly),
        }
    }

    /// Gets a mutable reference to the signal data payload. This "unwraps"
    /// the generic `Payload` into a `SignalData` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a signal data packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_signal_data_packet();
    /// let signal_data_mut = packet.payload_mut().signal_data_mut().unwrap();
    /// signal_data_mut.set_payload(&[1, 2, 3, 4]);
    /// assert_eq!(signal_data_mut.payload_size_bytes(), 4);
    /// ```
    pub fn signal_data_mut(&mut self) -> Result<&mut SignalData, VitaError> {
        match self {
            Payload::SignalData(p) => Ok(p),
            _ => Err(VitaError::SignalDataOnly),
        }
    }

    /// Gets a reference to the context payload. This "unwraps"
    /// the generic `Payload` into a `Context` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a context packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_context_packet();
    /// let context = packet.payload_mut().context_mut().unwrap();
    /// assert_eq!(context.bandwidth_hz(), None);
    /// ```
    pub fn context(&self) -> Result<&Context, VitaError> {
        match self {
            Payload::Context(p) => Ok(p),
            _ => Err(VitaError::ContextOnly),
        }
    }
    /// Gets a mutable reference to the context payload. This "unwraps"
    /// the generic `Payload` into a `Context` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a context packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_context_packet();
    /// let context_mut = packet.payload_mut().context_mut().unwrap();
    /// context_mut.set_bandwidth_hz(Some(8e6));
    /// assert_eq!(context_mut.bandwidth_hz(), Some(8e6));
    /// ```
    pub fn context_mut(&mut self) -> Result<&mut Context, VitaError> {
        match self {
            Payload::Context(p) => Ok(p),
            _ => Err(VitaError::ContextOnly),
        }
    }

    /// Gets a reference to the command payload. This "unwraps"
    /// the generic `Payload` into a `Command` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a command packet.
    pub fn command(&self) -> Result<&Command, VitaError> {
        match self {
            Payload::Command(p) => Ok(p),
            _ => Err(VitaError::CommandOnly),
        }
    }
    /// Gets a mutable reference to the command payload. This "unwraps"
    /// the generic `Payload` into a `Command` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a command packet.
    ///
    /// # Example
    /// ```
    /// use vita49::{prelude::*, ControlAckMode, ActionMode};
    /// let mut packet = Vrt::new_control_packet();
    /// let command_mut = packet.payload_mut().command_mut().unwrap();
    /// let mut cam = ControlAckMode::default();
    /// cam.set_action_mode(ActionMode::Execute);
    /// command_mut.set_cam(cam);
    /// assert_eq!(command_mut.cam().action_mode(), ActionMode::Execute);
    /// ```
    pub fn command_mut(&mut self) -> Result<&mut Command, VitaError> {
        match self {
            Payload::Command(p) => Ok(p),
            _ => Err(VitaError::CommandOnly),
        }
    }

    /// Gets the payload size in 32-bit words.
    pub fn size_words(&self) -> u16 {
        match self {
            Payload::SignalData(p) => p.size_words(),
            Payload::Context(p) => p.size_words(),
            Payload::Command(p) => p.size_words(),
        }
    }
}
