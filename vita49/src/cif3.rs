// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Defines fields and methods related to CIF3 (ANSI/VITA-49.2-2017 9.1).
Fields here are compatible with VITA 49.2 and later.
*/

use crate::command_prelude::*;
use crate::{ack_response::AckResponse, cif0::Cif0, cif7::Cif7Opts};
use deku::prelude::*;
use fixed::{types::extra::U6, FixedI16};
use vita49_macros::{
    ack_field, cif_basic, cif_field, cif_fields, cif_radix_masked, todo_cif_field,
};

/// Base data structure for the CIF3 single-bit indicators
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cif3(#[deku(assert = "(*field_0 & Cif3::UNSUPPORTED) == 0")] u32);

impl Cif3 {
    /// Bits selecting fields this crate cannot parse yet.
    ///
    /// Each term comes from the corresponding `todo_cif_field!` below, so the
    /// bit positions cannot drift apart. Add a term here whenever a new
    /// `todo_cif_field!` is introduced.
    const UNSUPPORTED: u32 = Self::UNSUPPORTED_AGE | Self::UNSUPPORTED_SHELF_LIFE;

    cif_field!(timestamp_details, 31);
    cif_field!(timestamp_skew, 30);
    // Bits 28-29 are reserved
    cif_field!(rise_time, 27);
    cif_field!(fall_time, 26);
    cif_field!(offset_time, 25);
    cif_field!(pulse_width, 24);
    cif_field!(period, 23);
    cif_field!(duration, 22);
    cif_field!(dwell, 21);
    cif_field!(jitter, 20);
    // Bits 18-19 are reserved
    todo_cif_field!(age, 17, 3);
    todo_cif_field!(shelf_life, 16, 3);
    // Bits 8-15 are reserved
    cif_field!(air_temp, 7);
    cif_field!(ground_temp, 6);
    cif_field!(humidity, 5);
    cif_field!(barometric_pressure, 4);
    cif_field!(sea_and_swell_state, 3);
    cif_field!(tropospheric_state, 2);
    cif_field!(network_id, 1);
    // Bit 0 is reserved

    fn empty(&self) -> bool {
        self.0 == 0
    }
}

#[cif_fields(cif3)]
pub struct Cif3Fields {
    // TODO: add full support
    timestamp_details: u64,
    timestamp_skew: i64,
    rise_time: i64,
    fall_time: i64,
    offset_time: i64,
    pulse_width: i64,
    period: i64,
    duration: i64,
    dwell: i64,
    jitter: i64,
    // TODO: add basic support
    age: u32,
    // TODO: add basic support
    shelf_life: u32,
    air_temp: i32,
    ground_temp: i32,
    humidity: u32,
    barometric_pressure: u32,
    sea_and_swell_state: u32,
    tropospheric_state: u32,
    network_id: u32,
}

#[cif_fields(cif3)]
pub struct Cif3AckFields {
    timestamp_details: AckResponse,
    timestamp_skew: AckResponse,
    rise_time: AckResponse,
    fall_time: AckResponse,
    offset_time: AckResponse,
    pulse_width: AckResponse,
    period: AckResponse,
    duration: AckResponse,
    dwell: AckResponse,
    jitter: AckResponse,
    age: AckResponse,
    shelf_life: AckResponse,
    air_temp: AckResponse,
    ground_temp: AckResponse,
    humidity: AckResponse,
    barometric_pressure: AckResponse,
    sea_and_swell_state: AckResponse,
    tropospheric_state: AckResponse,
    network_id: AckResponse,
}

/// Trait for common CIF3 manipulation methods. Used by Context and
/// Command packets.
#[rustfmt::skip]
pub trait Cif3Manipulators {
    /// Get a reference to the packet's CIF0 (indicators)
    fn cif0(&self) -> &Cif0;
    /// Get a mutable reference to the packet's CIF0 (indicators)
    fn cif0_mut(&mut self) -> &mut Cif0;
    /// Get a reference to the packet's CIF3 (indicators)
    fn cif3(&self) -> Option<&Cif3>;
    /// Get a mutable reference to the packet's CIF3 (indicators)
    fn cif3_mut(&mut self) -> &mut Option<Cif3>;
    /// Get a reference to the packet's CIF3 data fields
    fn cif3_fields(&self) -> Option<&Cif3Fields>;
    /// Get a mutable reference to the packet's CIF3 data fields
    fn cif3_fields_mut(&mut self) -> &mut Option<Cif3Fields>;

    // TODO: add full support
    cif_basic!(cif3, timestamp_details, timestamp_details, u64);
    cif_basic!(cif3, timestamp_skew, timestamp_skew, i64);
    cif_basic!(cif3, rise_time, rise_time, i64);
    cif_basic!(cif3, fall_time, fall_time, i64);
    cif_basic!(cif3, offset_time, offset_time, i64);
    cif_basic!(cif3, pulse_width, pulse_width, i64);
    cif_basic!(cif3, period, period, i64);
    cif_basic!(cif3, duration, duration, i64);
    cif_basic!(cif3, dwell, dwell, i64);
    cif_basic!(cif3, jitter, jitter, i64);
    // TODO: add basic support
    cif_basic!(cif3, age, age, u32);
    // TODO: add basic support
    cif_basic!(cif3, shelf_life, shelf_life, u32);
    cif_radix_masked!(cif3, air_temp, air_temp_c, f32, FixedI16::<U6>, i32, i16);
    cif_radix_masked!(cif3, ground_temp, ground_temp_c, f32, FixedI16::<U6>, i32, i16);
    // TODO: add full support
    cif_basic!(cif3, humidity, humidity, u32);
    // TODO: add full support
    cif_basic!(cif3, barometric_pressure, barometric_pressure, u32);
    // TODO: add full support
    cif_basic!(cif3, sea_and_swell_state, sea_and_swell_state, u32);
    // TODO: add full support
    cif_basic!(cif3, tropospheric_state, tropospheric_state, u32);
    cif_basic!(cif3, network_id, network_id, u32);
}

/// Shared trait for manipulating CIF3 ACK fields.
pub trait Cif3AckManipulators {
    /// Get a reference to the packet's WIF0 (indicators)
    fn wif0(&self) -> Option<&Cif0>;
    /// Get a mutable reference to the packet's WIF0 (indicators)
    fn wif0_mut(&mut self) -> &mut Option<Cif0>;
    /// Get a reference to the packet's WIF0 data fields
    fn wif0_fields(&self) -> Option<&Cif0AckFields>;
    /// Get a mutable reference to the packet's WIF0 data fields
    fn wif0_fields_mut(&mut self) -> &mut Option<Cif0AckFields>;

    /// Get a reference to the packet's EIF0 (indicators)
    fn eif0(&self) -> Option<&Cif0>;
    /// Get a mutable reference to the packet's EIF0 (indicators)
    fn eif0_mut(&mut self) -> &mut Option<Cif0>;
    /// Get a reference to the packet's EIF0 data fields
    fn eif0_fields(&self) -> Option<&Cif0AckFields>;
    /// Get a mutable reference to the packet's EIF0 data fields
    fn eif0_fields_mut(&mut self) -> &mut Option<Cif0AckFields>;

    /// Get a reference to the packet's WIF3 (indicators)
    fn wif3(&self) -> Option<&Cif3>;
    /// Get a mutable reference to the packet's WIF3 (indicators)
    fn wif3_mut(&mut self) -> &mut Option<Cif3>;
    /// Get a reference to the packet's WIF3 data fields
    fn wif3_fields(&self) -> Option<&Cif3AckFields>;
    /// Get a mutable reference to the packet's WIF3 data fields
    fn wif3_fields_mut(&mut self) -> &mut Option<Cif3AckFields>;

    /// Get a reference to the packet's EIF3 (indicators)
    fn eif3(&self) -> Option<&Cif3>;
    /// Get a mutable reference to the packet's EIF3 (indicators)
    fn eif3_mut(&mut self) -> &mut Option<Cif3>;
    /// Get a reference to the packet's EIF3 data fields
    fn eif3_fields(&self) -> Option<&Cif3AckFields>;
    /// Get a mutable reference to the packet's EIF3 data fields
    fn eif3_fields_mut(&mut self) -> &mut Option<Cif3AckFields>;

    ack_field!(3, timestamp_details);
    ack_field!(3, timestamp_skew);
    ack_field!(3, rise_time);
    ack_field!(3, fall_time);
    ack_field!(3, offset_time);
    ack_field!(3, pulse_width);
    ack_field!(3, period);
    ack_field!(3, duration);
    ack_field!(3, dwell);
    ack_field!(3, jitter);
    ack_field!(3, age);
    ack_field!(3, shelf_life);
    ack_field!(3, air_temp);
    ack_field!(3, ground_temp);
    ack_field!(3, humidity);
    ack_field!(3, barometric_pressure);
    ack_field!(3, sea_and_swell_state);
    ack_field!(3, tropospheric_state);
    ack_field!(3, network_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use deku::ctx::Endian;
    use std::io::Cursor;

    /// Parse a raw CIF3 word off the wire.
    fn read_cif3(raw: u32) -> Result<Cif3, DekuError> {
        let bytes = raw.to_be_bytes();
        let mut cursor = Cursor::new(&bytes[..]);
        let mut reader = Reader::new(&mut cursor);
        Cif3::from_reader_with_ctx(&mut reader, Endian::Big)
    }

    #[test]
    fn unsupported_bits_are_rejected_at_parse() {
        for bit in [17, 16] {
            assert!(
                read_cif3(1 << bit).is_err(),
                "CIF3 bit {bit} selects an unimplemented field and must not parse"
            );
        }
    }

    #[test]
    fn supported_bits_still_parse() {
        for bit in [31, 30, 26, 20, 7, 3] {
            assert!(
                read_cif3(1 << bit).is_ok(),
                "CIF3 bit {bit} is supported and must still parse"
            );
        }
    }

    #[test]
    fn unsupported_mask_covers_exactly_the_todo_fields() {
        assert_eq!(Cif3::UNSUPPORTED, (1 << 17) | (1 << 16));
    }
}
