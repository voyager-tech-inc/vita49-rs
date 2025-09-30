// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to command payloads
(ANSI/VITA-49.2-2017 section 8).
*/

use core::fmt;

use crate::{
    prelude::*, Ack, Cancellation, CommandPayload, Control, ControlAckMode, IdFormat, QueryAck,
};
use deku::prelude::*;

/// Main command payload structure.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, packet_header: &PacketHeader"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Command {
    /// Control acknowledgement mode.
    cam: ControlAckMode,
    /// Message ID.
    message_id: u32,
    /// Controllee ID.
    #[deku(cond = "cam.controllee_enabled() && cam.controllee_id_format() == IdFormat::Id32bit")]
    controllee_id: Option<u32>,
    /// Controllee UUID.
    #[deku(
        cond = "cam.controllee_enabled() && cam.controllee_id_format() == IdFormat::Uuid128bit"
    )]
    controllee_uuid: Option<u128>,
    /// Controller ID.
    #[deku(cond = "cam.controller_enabled() && cam.controller_id_format() == IdFormat::Id32bit")]
    controller_id: Option<u32>,
    /// Controller UUID.
    #[deku(
        cond = "cam.controller_enabled() && cam.controller_id_format() == IdFormat::Uuid128bit"
    )]
    controller_uuid: Option<u128>,
    #[deku(ctx = "&cam, packet_header")]
    command_payload: CommandPayload,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            cam: Default::default(),
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::Control(Control::default()),
        }
    }
}

impl Command {
    /// Create a new, empty control packet.
    pub fn new_control() -> Command {
        Command::default()
    }

    /// Create a new, empty cancellation packet.
    pub fn new_cancellation() -> Command {
        Self {
            cam: Default::default(),
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::Cancellation(Cancellation::default()),
        }
    }

    /// Create a new, empty validation ACK packet.
    pub fn new_validation_ack() -> Command {
        let mut cam = ControlAckMode::default();
        cam.set_validation();
        Self {
            cam,
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::ValidationAck(Ack::default()),
        }
    }

    /// Create a new, empty execution ACK packet.
    pub fn new_exec_ack() -> Command {
        let mut cam = ControlAckMode::default();
        cam.set_execution();
        Self {
            cam,
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::ExecAck(Ack::default()),
        }
    }

    /// Create a new, empty query ACK packet.
    pub fn new_query_ack() -> Command {
        let mut cam = ControlAckMode::default();
        cam.set_state();
        Self {
            cam,
            message_id: Default::default(),
            controllee_id: Default::default(),
            controllee_uuid: Default::default(),
            controller_id: Default::default(),
            controller_uuid: Default::default(),
            command_payload: CommandPayload::QueryAck(QueryAck::default()),
        }
    }

    /// Get the packet message ID.
    pub fn message_id(&self) -> u32 {
        self.message_id
    }

    /// Set the packet message ID.
    pub fn set_message_id(&mut self, message_id: u32) {
        self.message_id = message_id;
    }

    /// Get the packet's Control Ack Mode (CAM)
    pub fn cam(&self) -> ControlAckMode {
        self.cam
    }

    /// Set the packet's Control Ack Mode (CAM)
    /// # Example
    /// ```
    /// use vita49::{prelude::*, ControlAckMode, ActionMode};
    /// let mut packet = Vrt::new_control_packet();
    /// let command_mut = packet.payload_mut().command_mut().unwrap();
    /// let mut cam = ControlAckMode::default();
    /// cam.set_action_mode(ActionMode::Execute);
    /// command_mut.set_cam(cam);
    /// assert_eq!(command_mut.cam().action_mode(), ActionMode::Execute);
    /// ````
    pub fn set_cam(&mut self, mode: ControlAckMode) {
        self.cam = mode;
    }

    /// Get the controllee identifier.
    pub fn controllee_id(&self) -> Option<u32> {
        self.controllee_id
    }
    /// Sets the controllee identifier. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controllee_uuid` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controllee_id(&mut self, id: Option<u32>) -> Result<(), VitaError> {
        if id.is_some() && self.controllee_uuid.is_some() {
            return Err(VitaError::TriedIdWhenUuidSet);
        }
        self.controllee_id = id;
        if id.is_some() {
            self.cam.enable_controllee();
            self.cam.set_controllee_id_format(IdFormat::Id32bit);
        } else if self.controllee_uuid.is_none() {
            self.cam.disable_controllee();
        }
        Ok(())
    }

    /// Get the controller identifier.
    pub fn controller_id(&self) -> Option<u32> {
        self.controller_id
    }
    /// Sets the controller identifier. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controller_uuid` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controller_id(&mut self, id: Option<u32>) -> Result<(), VitaError> {
        if id.is_some() && self.controller_uuid.is_some() {
            return Err(VitaError::TriedIdWhenUuidSet);
        }
        self.controller_id = id;
        if id.is_some() {
            self.cam.enable_controller();
            self.cam.set_controller_id_format(IdFormat::Id32bit);
        } else if self.controller_uuid.is_none() {
            self.cam.disable_controller();
        }
        Ok(())
    }

    /// Get the controllee UUID.
    pub fn controllee_uuid(&self) -> Option<u128> {
        self.controllee_uuid
    }
    /// Sets the controllee UUID. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controllee_id` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controllee_uuid(&mut self, uuid: Option<u128>) -> Result<(), VitaError> {
        if uuid.is_some() && self.controllee_id.is_some() {
            return Err(VitaError::TriedUuidWhenIdSet);
        }
        self.controllee_uuid = uuid;
        if uuid.is_some() {
            self.cam.enable_controllee();
            self.cam.set_controllee_id_format(IdFormat::Uuid128bit);
        } else if self.controllee_id.is_none() {
            self.cam.disable_controllee();
        }
        Ok(())
    }

    /// Get the controller UUID.
    pub fn controller_uuid(&self) -> Option<u128> {
        self.controller_uuid
    }
    /// Sets the controller UUID. If `None` is passed, the field
    /// will be unset.
    ///
    /// # Errors
    /// If this function is called while the `controller_id` field is set,
    /// an error will be returned as these fields are mutually exclusive.
    pub fn set_controller_uuid(&mut self, uuid: Option<u128>) -> Result<(), VitaError> {
        if uuid.is_some() && self.controller_id.is_some() {
            return Err(VitaError::TriedUuidWhenIdSet);
        }
        self.controller_uuid = uuid;
        if uuid.is_some() {
            self.cam.enable_controller();
            self.cam.set_controller_id_format(IdFormat::Uuid128bit);
        } else if self.controller_id.is_none() {
            self.cam.disable_controller();
        }
        Ok(())
    }

    /// Get a reference to the underlying command payload enumeration.
    pub fn payload(&self) -> &CommandPayload {
        &self.command_payload
    }

    /// Get a mutable reference to the underlying command payload enumeration.
    pub fn payload_mut(&mut self) -> &mut CommandPayload {
        &mut self.command_payload
    }

    /// Get the size of the command packet (in 32-bit words).
    pub fn size_words(&self) -> u16 {
        let mut ret = self.cam.size_words();
        ret += 1; // message_id
        if self.controllee_id.is_some() {
            ret += 1;
        } else if self.controllee_uuid.is_some() {
            ret += 4;
        }
        if self.controller_id.is_some() {
            ret += 1;
        } else if self.controller_uuid.is_some() {
            ret += 4;
        }
        ret += self.command_payload.size_words();
        ret
    }
}

impl TryFrom<Payload> for Command {
    type Error = Payload;

    fn try_from(value: Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::Command(c) => Ok(c),
            a => Err(a),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cam)?;
        writeln!(f, "Message ID: {:x}", self.message_id)?;
        if let Some(cid) = self.controllee_id {
            writeln!(f, "Controllee ID: {cid:x}")?;
        }
        if let Some(cuuid) = self.controllee_uuid {
            writeln!(f, "Controllee UUID: {cuuid:x}")?;
        }
        if let Some(cid) = self.controller_id {
            writeln!(f, "Controller ID: {cid:x}")?;
        }
        if let Some(cuuid) = self.controller_uuid {
            writeln!(f, "Controller UUID: {cuuid:x}")?;
        }
        match &self.command_payload {
            CommandPayload::Control(p) => write!(f, "{p}")?,
            CommandPayload::Cancellation(p) => write!(f, "{p}")?,
            CommandPayload::ValidationAck(p) => write!(f, "Validation {p}")?,
            CommandPayload::ExecAck(p) => write!(f, "Execution {p}")?,
            CommandPayload::QueryAck(p) => write!(f, "{p}")?,
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::{ActionMode, ControlAckMode, IdFormat, Tsf, Tsi};

    #[test]
    fn create_control_packet() {
        let mut packet = Vrt::new_control_packet();
        packet.set_stream_id(Some(0xDEADBEEF));
        packet.set_integer_timestamp(Some(0), Tsi::Utc).unwrap();
        packet
            .set_fractional_timestamp(Some(0), Tsf::SampleCount)
            .unwrap();
        let command = packet.payload_mut().command_mut().unwrap();
        command.set_message_id(123);
        let mut cam = ControlAckMode::default();
        cam.enable_controllee();
        cam.enable_controller();
        cam.set_controllee_id_format(IdFormat::Id32bit);
        cam.set_controller_id_format(IdFormat::Uuid128bit);
        cam.set_action_mode(ActionMode::Execute);
        cam.set_partial_packet_impl_permitted();
        cam.set_warnings_permitted();
        cam.set_validation();
        cam.set_warning();
        cam.set_error();
        command.set_cam(cam);
        command.controllee_id = Some(123);
        command.controller_uuid = Some(321);
    }
}
