// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![deny(missing_docs)]
#![deny(unstable_features, unused_import_braces, unreachable_pub)]
// TODO: remove after 0.1.0 release
#![allow(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::unescaped_backticks)]
#![forbid(unsafe_code)]

mod ack;
mod ack_response;
mod cancellation;
mod cif0;
mod cif1;
mod cif2;
mod cif3;
mod cif7;
mod class_id;
mod command;
mod command_payload;
mod context;
mod context_association_lists;
mod control;
mod control_ack_mode;
mod device_id;
mod ecef_ephemeris;
mod errors;
mod formatted_gps;
mod gain;
mod gps_ascii;
mod packet_header;
mod payload;
mod query_ack;
mod signal_data;
mod spectrum;
mod threshold;
mod trailer;
mod vrt;

// Public exports
pub use crate::ack::{Ack, AckLevel};
pub use crate::ack_response::AckResponse;
pub use crate::cancellation::Cancellation;
pub use crate::cif0::*;
pub use crate::cif1::*;
pub use crate::cif2::*;
pub use crate::cif3::*;
pub use crate::cif7::Cif7;
pub use crate::class_id::ClassIdentifier;
pub use crate::command::Command;
pub use crate::command_payload::CommandPayload;
pub use crate::context::Context;
pub use crate::context_association_lists::ContextAssociationLists;
pub use crate::control::Control;
pub use crate::control_ack_mode::*;
pub use crate::device_id::DeviceId;
pub use crate::ecef_ephemeris::EcefEphemeris;
pub use crate::errors::VitaError;
pub use crate::formatted_gps::FormattedGps;
pub use crate::gain::Gain;
pub use crate::gps_ascii::GpsAscii;
pub use crate::packet_header::*;
pub use crate::payload::Payload;
pub use crate::query_ack::QueryAck;
pub use crate::signal_data::SignalData;
pub use crate::spectrum::*;
pub use crate::threshold::Threshold;
pub use crate::trailer::{SampleFrameIndicator, Trailer};
pub use crate::vrt::Vrt;

/// Standard imports for the most commonly used structures and
/// traits in the vita49 crate.
pub mod prelude {
    pub use crate::cif0::{Cif0, Cif0Fields, Cif0Manipulators};
    pub use crate::cif1::{Cif1, Cif1Fields, Cif1Manipulators};
    pub use crate::cif2::{Cif2, Cif2Fields, Cif2Manipulators};
    pub use crate::cif3::{Cif3, Cif3Fields, Cif3Manipulators};
    pub use crate::cif7::Cif7;
    pub use crate::class_id::ClassIdentifier;
    pub use crate::context::Context;
    pub use crate::errors::VitaError;
    pub use crate::packet_header::*;
    pub use crate::payload::Payload;
    pub use crate::signal_data::SignalData;
    pub use crate::vrt::Vrt;
    pub use deku::writer::Writer;
    pub use deku::{DekuContainerRead, DekuContainerWrite, DekuReader, DekuWriter};
}

/// Standard imports for programs that interact with VITA 49.2 command/control/ack
/// packets.
pub mod command_prelude {
    pub use crate::cif0::{Cif0AckFields, Cif0AckManipulators};
    pub use crate::cif1::{Cif1AckFields, Cif1AckManipulators};
    pub use crate::{
        Ack, AckLevel, AckResponse, ActionMode, Cancellation, Command, CommandPayload, Control,
        ControlAckMode, QueryAck,
    };
}
