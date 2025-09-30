// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{prelude::*, Ack, Cancellation, Control, ControlAckMode, QueryAck};
use deku::prelude::*;

/// Command payload enumeration. Command payloads can take several different forms depending
/// on various header and CAM fields. Basically, here's the breakdown:
///
/// ```text
///                              ┌──────────────────┐                                          
///                              │                  │                                          
///                    ┌─────────┤  Command Packet  ├─────────┐                                
///                    │         │      Types       │         │                                
///                    │         │                  │         │                                
///           ┌────────▼───────┐ └──────────────────┘  ┌──────▼──────┐                         
///           │                │                       │             │                         
///         ┌─┤ Control Packet ├──┐                  ┌─┤  ACK Packet ├──┬──────────────┐       
///         │ │     Types      │  │                  │ │    Types    │  │              │       
///         │ │                │  │                  │ │             │  │              │       
///         │ └────────────────┘  │                  │ └─────────────┘  │              │       
///         │                     │                  │                  │              │       
/// ┌───────▼────────┐   ┌────────▼───────┐    ┌─────▼───────────┐ ┌────▼─────┐  ┌─────▼──────┐
/// │    Control     │   │  Cancellation  │    │  Validation ACK │ │ Exec ACK │  │ Query ACK  │
/// │     Packet     │   │     Packet     │    │      Packet     │ │  Packet  │  │   Packet   │
/// └────────────────┘   └────────────────┘    └─────────────────┘ └──────────┘  └────────────┘
/// ```
///
/// For the actual packet types, here are some attributes:
/// 1. Control Packet
///    - Includes all CIF indicators
///    - In Action Mode 0, will NOT include CIF fields
///    - In other Action Modes, WILL include CIF fields
/// 2. Cancellation Packet
///    - Only includes CIF indicator fields (no real data fields)
/// 3. Validation ACK
///    - Can include warning indicators/fields and error indicators/fields
/// 4. Exec ACK
///    - Can include warning indicators/fields and error indicators/fields
/// 5. Query ACK
///    - Very similar to a context packet, this will include all CIF indicators and fields.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, cam: &ControlAckMode, packet_header: &PacketHeader",
    id = "CommandPayload::derive_type(cam, packet_header)"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CommandPayload {
    /// Payload for a control packet.
    #[deku(id = "CommandPayload::Control(_)")]
    Control(Control),
    /// Payload for a cancellation packet.
    #[deku(id = "CommandPayload::Cancellation(_)")]
    Cancellation(Cancellation),
    /// Payload for a validation ACK packet.
    #[deku(id = "CommandPayload::ValidationAck(_)")]
    ValidationAck(#[deku(ctx = "cam")] Ack),
    /// Payload for a execution ACK packet.
    #[deku(id = "CommandPayload::ExecAck(_)")]
    ExecAck(#[deku(ctx = "cam")] Ack),
    /// Payload for a query ACK packet.
    #[deku(id = "CommandPayload::QueryAck(_)")]
    QueryAck(QueryAck),
}

impl CommandPayload {
    /// Determine the type of command payload based on CAM field and VRT packet header.
    fn derive_type(cam: &ControlAckMode, packet_header: &PacketHeader) -> CommandPayload {
        if packet_header.is_ack_packet().unwrap() {
            if [cam.validation(), cam.execution(), cam.state()]
                .iter()
                .filter(|&x| *x)
                .count()
                != 1
            {
                panic!("CAM field in ACK packet does not exclusively select one of Validation, Exec, or Query");
            }
            if cam.validation() {
                CommandPayload::ValidationAck(Ack::default())
            } else if cam.execution() {
                CommandPayload::ExecAck(Ack::default())
            } else if cam.state() {
                CommandPayload::QueryAck(QueryAck::default())
            } else {
                unreachable!()
            }
        } else if packet_header.is_cancellation_packet().unwrap() {
            CommandPayload::Cancellation(Cancellation::default())
        } else {
            CommandPayload::Control(Control::default())
        }
    }

    /// Get the size of the command payload (in 32-bit words).
    pub fn size_words(&self) -> u16 {
        match self {
            CommandPayload::Control(p) => p.size_words(),
            CommandPayload::Cancellation(p) => p.size_words(),
            CommandPayload::ValidationAck(p) => p.size_words(),
            CommandPayload::ExecAck(p) => p.size_words(),
            CommandPayload::QueryAck(p) => p.size_words(),
        }
    }

    /// Gets a reference to the control payload. This "unwraps"
    /// the generic `CommandPayload` into a `Control` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a control packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let packet = Vrt::new_control_packet();
    /// let command = packet.payload().command().unwrap();
    /// let control = command.payload().control().unwrap();
    /// assert_eq!(control.bandwidth_hz(), None);
    /// ```
    pub fn control(&self) -> Result<&Control, VitaError> {
        match self {
            CommandPayload::Control(p) => Ok(p),
            _ => Err(VitaError::ControlOnly),
        }
    }

    /// Gets a mutable reference to the control payload. This "unwraps"
    /// the generic `CommandPayload` into a `Control` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a control packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_control_packet();
    /// let mut command = packet.payload_mut().command_mut().unwrap();
    /// let mut control = command.payload_mut().control_mut().unwrap();
    /// control.set_bandwidth_hz(Some(64e6));
    /// assert_eq!(control.bandwidth_hz(), Some(64e6));
    /// ```
    pub fn control_mut(&mut self) -> Result<&mut Control, VitaError> {
        match self {
            CommandPayload::Control(p) => Ok(p),
            _ => Err(VitaError::ControlOnly),
        }
    }

    /// Gets a reference to the cancellation payload. This "unwraps"
    /// the generic `CommandPayload` into a `Cancellation` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a cancellation packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let packet = Vrt::new_cancellation_packet();
    /// let command = packet.payload().command().unwrap();
    /// let cancel = command.payload().cancellation().unwrap();
    /// assert!(!cancel.cif0().bandwidth());
    /// ```
    pub fn cancellation(&self) -> Result<&Cancellation, VitaError> {
        match self {
            CommandPayload::Cancellation(p) => Ok(p),
            _ => Err(VitaError::CancellationOnly),
        }
    }

    /// Gets a reference to the cancellation payload. This "unwraps"
    /// the generic `CommandPayload` into a `Cancellation` payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a cancellation packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// let mut packet = Vrt::new_cancellation_packet();
    /// let command = packet.payload_mut().command_mut().unwrap();
    /// let cancel = command.payload_mut().cancellation_mut().unwrap();
    /// cancel.cif0_mut().set_bandwidth();
    /// assert!(cancel.cif0().bandwidth());
    /// ```
    pub fn cancellation_mut(&mut self) -> Result<&mut Cancellation, VitaError> {
        match self {
            CommandPayload::Cancellation(p) => Ok(p),
            _ => Err(VitaError::CancellationOnly),
        }
    }

    /// Gets a reference to the validation ack payload. This "unwraps"
    /// the generic `CommandPayload` into an [`Ack`] payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a validation ack packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let packet = Vrt::new_validation_ack_packet();
    /// let command = packet.payload().command().unwrap();
    /// let ack = command.payload().validation_ack().unwrap();
    /// assert!(ack.bandwidth().is_none());
    /// ```
    pub fn validation_ack(&self) -> Result<&Ack, VitaError> {
        match self {
            CommandPayload::ValidationAck(p) => Ok(p),
            _ => Err(VitaError::ValidationAckOnly),
        }
    }

    /// Gets a mutable reference to the validation ack payload. This "unwraps"
    /// the generic `CommandPayload` into an [`Ack`] payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a validation ack packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let mut packet = Vrt::new_validation_ack_packet();
    /// let command = packet.payload_mut().command_mut().unwrap();
    /// let ack = command.payload_mut().validation_ack_mut().unwrap();
    /// let mut response = AckResponse::default();
    /// response.set_param_out_of_range();
    /// ack.set_bandwidth(AckLevel::Error, Some(response));
    /// assert!(ack.bandwidth().is_some())
    /// ```
    pub fn validation_ack_mut(&mut self) -> Result<&mut Ack, VitaError> {
        match self {
            CommandPayload::ValidationAck(p) => Ok(p),
            _ => Err(VitaError::ValidationAckOnly),
        }
    }

    /// Gets a reference to the exec ack payload. This "unwraps"
    /// the generic `CommandPayload` into an [`Ack`] payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a exec ack packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let packet = Vrt::new_exec_ack_packet();
    /// let command = packet.payload().command().unwrap();
    /// let ack = command.payload().exec_ack().unwrap();
    /// assert!(ack.bandwidth().is_none());
    /// ```
    pub fn exec_ack(&self) -> Result<&Ack, VitaError> {
        match self {
            CommandPayload::ExecAck(p) => Ok(p),
            _ => Err(VitaError::ExecAckOnly),
        }
    }

    /// Gets a mutable reference to the exec ack payload. This "unwraps"
    /// the generic `CommandPayload` into an [`Ack`] payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a exec ack packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let mut packet = Vrt::new_exec_ack_packet();
    /// let command = packet.payload_mut().command_mut().unwrap();
    /// let ack = command.payload_mut().exec_ack_mut().unwrap();
    /// let mut response = AckResponse::default();
    /// response.set_param_out_of_range();
    /// ack.set_bandwidth(AckLevel::Error, Some(response));
    /// assert!(ack.bandwidth().is_some())
    /// ```
    pub fn exec_ack_mut(&mut self) -> Result<&mut Ack, VitaError> {
        match self {
            CommandPayload::ExecAck(p) => Ok(p),
            _ => Err(VitaError::ExecAckOnly),
        }
    }

    /// Gets a reference to the query ack payload. This "unwraps"
    /// the generic `CommandPayload` into an [`QueryAck`] payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a query ack packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let packet = Vrt::new_query_ack_packet();
    /// let command = packet.payload().command().unwrap();
    /// let ack = command.payload().query_ack().unwrap();
    /// assert!(ack.bandwidth_hz().is_none());
    /// ```
    pub fn query_ack(&self) -> Result<&QueryAck, VitaError> {
        match self {
            CommandPayload::QueryAck(p) => Ok(p),
            _ => Err(VitaError::QueryAckOnly),
        }
    }

    /// Gets a mutable reference to the query ack payload. This "unwraps"
    /// the generic `CommandPayload` into an [`QueryAck`] payload.
    ///
    /// # Errors
    /// This function will return an error if run on a packet other
    /// than a query ack packet.
    ///
    /// # Example
    /// ```
    /// use vita49::prelude::*;
    /// use vita49::command_prelude::*;
    /// let mut packet = Vrt::new_query_ack_packet();
    /// let command = packet.payload_mut().command_mut().unwrap();
    /// let ack = command.payload_mut().query_ack_mut().unwrap();
    /// ack.set_bandwidth_hz(Some(100e6));
    /// assert!(ack.bandwidth_hz().is_some())
    /// ```
    pub fn query_ack_mut(&mut self) -> Result<&mut QueryAck, VitaError> {
        match self {
            CommandPayload::QueryAck(p) => Ok(p),
            _ => Err(VitaError::QueryAckOnly),
        }
    }
}
