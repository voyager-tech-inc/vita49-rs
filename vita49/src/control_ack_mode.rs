// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the Control Acknowledgement Mode
(CAM) field (ANSI/VITA-49.2-2017 section 8.2.1).
*/

use core::fmt;

use deku::prelude::*;

/// Base CAM field data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlAckMode(#[deku(assert = "ControlAckMode::validate_cam_raw(*field_0)")] u32);

/// Identification format (128-bit UUID or 32-bit ID).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdFormat {
    /// 128-bit UUID
    Uuid128bit,
    /// 32-bit ID
    Id32bit,
}

/// Control action mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionMode {
    /// No action should be taken.
    NoAction,
    /// A dry run of the action.
    DryRun,
    /// The action should execute.
    Execute,
    // All other values are reserved
}

/// Timing control mode.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimingControlMode {
    /// Ignore the timestamp.
    IgnoreTimestamp,
    /// Execute according to the device timestamp.
    DeviceExecutionOnly,
    /// Allow late or specified execution.
    LateAndSpecifiedExecution,
    /// Allow early or specified execution.
    EarlyAndSpecifiedExecution,
    /// Allow early or late execution.
    PermittedEarlyOrLateExecution,
}

impl ControlAckMode {
    pub(crate) fn validate_cam_raw(val: u32) -> bool {
        let action_mode_bits = (val >> 23) & 0b11;
        let timing_control_bits = (val >> 12) & 0b111;
        action_mode_bits != 0b11 && timing_control_bits <= 0b100
    }

    /// Generate a new Control Ack Mode field that's zeroed out.
    pub fn new(&self) -> ControlAckMode {
        ControlAckMode::default()
    }

    /// Returns the size of the CAM field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (std::mem::size_of_val(self) / std::mem::size_of::<u32>()) as u16
    }

    /// Returns true if a bit in the field is set, false if not.
    fn bit_is_set(&self, bit: u32) -> bool {
        self.0 & (1 << bit) != 0
    }

    /// Sets the bit.
    fn set_bit(&mut self, bit: u32) {
        self.0 |= 1 << bit;
    }

    /// Unsets the bit.
    fn unset_bit(&mut self, bit: u32) {
        self.0 &= !(1 << bit);
    }

    /// Returns true if the controllee enable bit is set, false if not.
    pub fn controllee_enabled(&self) -> bool {
        self.bit_is_set(31)
    }
    /// Sets the controllee enable bit.
    pub(crate) fn enable_controllee(&mut self) {
        self.set_bit(31);
    }
    /// Unsets the controllee enable bit.
    pub(crate) fn disable_controllee(&mut self) {
        self.unset_bit(31);
    }

    /// Returns the controllee ID format.
    pub(crate) fn controllee_id_format(&self) -> IdFormat {
        if self.bit_is_set(30) {
            IdFormat::Uuid128bit
        } else {
            IdFormat::Id32bit
        }
    }
    /// Sets the controllee ID format.
    pub(crate) fn set_controllee_id_format(&mut self, format: IdFormat) {
        match format {
            IdFormat::Id32bit => self.unset_bit(30),
            IdFormat::Uuid128bit => self.set_bit(30),
        }
    }

    /// Returns true if the controller enable bit is set, false if not.
    pub fn controller_enabled(&self) -> bool {
        self.bit_is_set(29)
    }
    /// Sets the controller enable bit.
    pub(crate) fn enable_controller(&mut self) {
        self.set_bit(29);
    }
    /// Unsets the controller enable bit.
    pub(crate) fn disable_controller(&mut self) {
        self.unset_bit(29);
    }

    /// Returns the controller ID format.
    pub(crate) fn controller_id_format(&self) -> IdFormat {
        if self.bit_is_set(28) {
            IdFormat::Uuid128bit
        } else {
            IdFormat::Id32bit
        }
    }
    /// Sets the controller ID format.
    pub(crate) fn set_controller_id_format(&mut self, format: IdFormat) {
        match format {
            IdFormat::Id32bit => self.unset_bit(28),
            IdFormat::Uuid128bit => self.set_bit(28),
        }
    }

    /// Returns true if partial packet implementations are permitted, false if not.
    pub fn partial_packet_impl_permitted(&self) -> bool {
        self.bit_is_set(27)
    }
    /// Sets the partial packet impl permitted bit.
    pub fn set_partial_packet_impl_permitted(&mut self) {
        self.set_bit(27);
    }
    /// Unsets the partial packet impl permitted bit.
    pub fn unset_partial_packet_impl_permitted(&mut self) {
        self.unset_bit(27);
    }

    /// Returns true if packet warnings are permitted, false if not.
    pub fn warnings_permitted(&self) -> bool {
        self.bit_is_set(26)
    }
    /// Sets the warnings permitted bit.
    pub fn set_warnings_permitted(&mut self) {
        self.set_bit(26);
    }
    /// Unsets the warnings permitted bit.
    pub fn unset_warnings_permitted(&mut self) {
        self.unset_bit(26);
    }

    /// Returns true if packet errors are permitted, false if not.
    pub fn errors_permitted(&self) -> bool {
        self.bit_is_set(25)
    }
    /// Sets the errors permitted bit.
    pub fn set_errors_permitted(&mut self) {
        self.set_bit(25);
    }
    /// Unsets the errors permitted bit.
    pub fn unset_errors_permitted(&mut self) {
        self.unset_bit(25);
    }

    /// Returns the action mode.
    pub fn action_mode(&self) -> ActionMode {
        let mode_bits = (self.0 >> 23) & 0b11;
        match mode_bits {
            0b00 => ActionMode::NoAction,
            0b01 => ActionMode::DryRun,
            0b10 => ActionMode::Execute,
            _ => unreachable!(),
        }
    }

    /// Sets the action mode.
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
    /// ````
    pub fn set_action_mode(&mut self, mode: ActionMode) {
        let val = match mode {
            ActionMode::NoAction => 0b00,
            ActionMode::DryRun => 0b01,
            ActionMode::Execute => 0b10,
        };
        self.0 = (self.0 & !(0b11 << 23)) | (val << 23);
    }

    /// Return NACK-only mode.
    /// When true: Provide AckV and/or AckX variants of
    /// Acknowledge packet ONLY when Warnings or Errors
    /// have occurred (referred to as "NACK"), as per the AckV
    /// and AckX bits.
    /// When false: Provide AckV and/or AckX packets in all cases
    /// when requested per the AckV and AckX bits.
    pub fn nack_only(&self) -> bool {
        self.bit_is_set(22)
    }
    /// Sets the NACK only bit.
    pub fn set_nack_only(&mut self) {
        self.set_bit(22);
    }
    /// Unsets the errors bit.
    pub fn unset_nack_only(&mut self) {
        self.unset_bit(22);
    }

    /// Returns true if request validation ACK is requested, false if not.
    pub fn validation(&self) -> bool {
        self.bit_is_set(20)
    }
    /// Sets the validation bit.
    pub fn set_validation(&mut self) {
        self.set_bit(20);
    }
    /// Unsets the validation bit.
    pub fn unset_validation(&mut self) {
        self.unset_bit(20);
    }

    /// Returns true if request execution ACK is requested, false if not.
    pub fn execution(&self) -> bool {
        self.bit_is_set(19)
    }
    /// Sets the execution bit.
    pub fn set_execution(&mut self) {
        self.set_bit(19);
    }
    /// Unsets the execution bit.
    pub fn unset_execution(&mut self) {
        self.unset_bit(19);
    }

    /// Returns true if request query-state ACK is requested, false if not.
    pub fn state(&self) -> bool {
        self.bit_is_set(18)
    }
    /// Sets the state bit.
    pub fn set_state(&mut self) {
        self.set_bit(18);
    }
    /// Unsets the state bit.
    pub fn unset_state(&mut self) {
        self.unset_bit(18);
    }

    /// For ACK packets, returns true if warnings are reported in the ACK, false if not.
    ///
    /// For Control packets, returns true if warnings are requested in the ACK, false if not.
    pub fn warning(&self) -> bool {
        self.bit_is_set(17)
    }
    /// Sets the warning bit.
    pub fn set_warning(&mut self) {
        self.set_bit(17);
    }
    /// Unsets the warning bit.
    pub fn unset_warning(&mut self) {
        self.unset_bit(17);
    }

    /// For ACK packets, returns true if errors are reported in the ACK, false if not.
    ///
    /// For Control packets, returns true if errors are requested in the ACK, false if not.
    pub fn error(&self) -> bool {
        self.bit_is_set(16)
    }
    /// Sets the error bit.
    pub fn set_error(&mut self) {
        self.set_bit(16);
    }
    /// Unsets the error bit.
    pub fn unset_error(&mut self) {
        self.unset_bit(16);
    }

    /// Returns timing control mode.
    pub fn timing_control(&self) -> TimingControlMode {
        let ctl_bits = (self.0 >> 12) & 0b111;
        match ctl_bits {
            0b000 => TimingControlMode::IgnoreTimestamp,
            0b001 => TimingControlMode::DeviceExecutionOnly,
            0b010 => TimingControlMode::LateAndSpecifiedExecution,
            0b011 => TimingControlMode::EarlyAndSpecifiedExecution,
            0b100 => TimingControlMode::PermittedEarlyOrLateExecution,
            _ => panic!("invalid timing control mode"),
        }
    }
    /// Sets the timing control mode.
    pub fn set_timing_control(&mut self, mode: TimingControlMode) {
        let val = (mode as u32) & 0b111;
        self.0 = (self.0 & !(0b111 << 12)) | (val << 12);
    }

    /// Returns true if partial action was taken/should be taken, false if not.
    pub fn partial_action_taken(&self) -> bool {
        self.bit_is_set(11)
    }
    /// Sets the partial action taken bit.
    pub fn set_partial_action_taken(&mut self) {
        self.set_bit(11);
    }
    /// Unsets the partial action taken bit.
    pub fn unset_partial_action_taken(&mut self) {
        self.unset_bit(11);
    }

    /// Returns true if action was scheduled/executed, false if not.
    pub fn action_scheduled_or_executed(&self) -> bool {
        self.bit_is_set(10)
    }
    /// Sets the action scheduled or executed bit.
    pub fn set_action_scheduled_or_executed(&mut self) {
        self.set_bit(10);
    }
    /// Unsets the action scheduled or executed bit.
    pub fn unset_action_scheduled_or_executed(&mut self) {
        self.unset_bit(10);
    }
}

impl fmt::Display for ControlAckMode {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CAM:")?;
        writeln!(f, "  Controllee enabled: {}", self.controllee_enabled())?;
        writeln!(f, "  Controllee ID format: {:?}", self.controllee_id_format())?;
        writeln!(f, "  Controller enabled: {}", self.controller_enabled())?;
        writeln!(f, "  Controller ID format: {:?}", self.controller_id_format())?;
        writeln!(f, "  Partial packet impl permitted: {}", self.partial_packet_impl_permitted())?;
        writeln!(f, "  Warnings permitted: {}", self.warnings_permitted())?;
        writeln!(f, "  Errors permitted: {}", self.errors_permitted())?;
        writeln!(f, "  Action mode: {:?}", self.action_mode())?;
        writeln!(f, "  NACK only: {}", self.nack_only())?;
        writeln!(f, "  Validation: {}", self.validation())?;
        writeln!(f, "  Execution: {}", self.execution())?;
        writeln!(f, "  State: {}", self.state())?;
        writeln!(f, "  Warning: {}", self.warning())?;
        writeln!(f, "  Error: {}", self.error())?;
        writeln!(f, "  Timing control: {:?}", self.timing_control())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_action_mode_fails_deku_parsing() {
        // Bit 24:23 = 0b11 is invalid action mode
        let invalid_cam_bytes: [u8; 4] = [0b0000_0001, 0b1000_0000, 0, 0];
        let mut cursor = deku::no_std_io::Cursor::new(&invalid_cam_bytes[..]);
        let mut reader = deku::reader::Reader::new(&mut cursor);
        let res = ControlAckMode::from_reader_with_ctx(&mut reader, deku::ctx::Endian::Big);
        assert!(matches!(res, Err(deku::DekuError::Assertion(_))));
    }

    #[test]
    fn invalid_timing_control_fails_deku_parsing() {
        // Bits 14:12 > 0b100 (e.g. 0b101 = 5) is invalid timing control
        let invalid_cam_bytes: [u8; 4] = [0, 0, 0b0101_0000, 0];
        let mut cursor = deku::no_std_io::Cursor::new(&invalid_cam_bytes[..]);
        let mut reader = deku::reader::Reader::new(&mut cursor);
        let res = ControlAckMode::from_reader_with_ctx(&mut reader, deku::ctx::Endian::Big);
        assert!(matches!(res, Err(deku::DekuError::Assertion(_))));
    }
}
