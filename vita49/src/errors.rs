// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Error types/enumerations for the `vita49` crate.
*/

use thiserror::Error;

/// Generic `vita49` crate error enumeration.
#[derive(Error, Debug)]
pub enum VitaError {
    /// Indicates a payload that requires an even number of 32-bit words
    /// was given something else.
    #[error("payload must be an even number of 32-bit words")]
    PayloadUneven32BitWords,
    /// Error given when a function that can only operate on signal
    /// data packets is executed on something else.
    #[error("function can only run on signal data packets")]
    SignalDataOnly,
    /// Error given when a function that can only operate on context
    /// packets is executed on something else.
    #[error("function can only run on context packets")]
    ContextOnly,
    /// Error given when a function that can only operate on command
    /// packets is executed on something else.
    #[error("function can only run on command packets")]
    CommandOnly,
    /// Error given when a function that can only operate on control
    /// sub-packets is executed on something else.
    #[error("function can only run on control command packets")]
    ControlOnly,
    /// Error given when a function that can only operate on cancellation
    /// sub-packets is executed on something else.
    #[error("function can only run on cancellation command packets")]
    CancellationOnly,
    /// Error given when a function that can only operate on validation ACK
    /// sub-packets is executed on something else.
    #[error("function can only run on validation ACK command packets")]
    ValidationAckOnly,
    /// Error given when a function that can only operate on execution ACK
    /// sub-packets is executed on something else.
    #[error("function can only run on execution ACK command packets")]
    ExecAckOnly,
    /// Error given when a function that can only operate on query ACK
    /// sub-packets is executed on something else.
    #[error("function can only run on query ACK command packets")]
    QueryAckOnly,
    /// Error given when attempting to set a timestamp field with a
    /// Tsi or Tsf mode that doesn't make sense.
    #[error("attempted to set timestamp field with Tsi/Tsf mode that doesn't make sense")]
    TimestampModeMismatch,
    /// Error given when attempting to use a controller/controllee ID
    /// while the UUID is set. ID and UUID are mutually exclusive.
    #[error("attempted to set controllee/controller ID field when UUID field is set")]
    TriedIdWhenUuidSet,
    /// Error given when attempting to use a controller/controllee UUID
    /// while the ID is set. ID and UUID are mutually exclusive.
    #[error("attempted to set controllee/controller UUID field when ID field is set")]
    TriedUuidWhenIdSet,
    /// Error given when attempting to use an out-of-range value.
    #[error("out of range")]
    OutOfRange,
    /// Error given when trying to set a reserved value.
    #[error("attempted to set reserved field")]
    ReservedField,
    /// Error given when GPS ASCII payload bytes are not valid UTF-8.
    #[error("GPS ASCII payload is not valid UTF-8")]
    InvalidAscii(#[from] std::string::FromUtf8Error),
}
