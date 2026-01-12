// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Defines fields and methods related to CIF0 (ANSI/VITA-49.2-2017 9.1).
Fields here are compatible with VITA 49.0 and later.
*/

use core::fmt;

use crate::ack::AckLevel;
use crate::ack_response::AckResponse;
use crate::device_id::DeviceId;
use crate::{
    cif7::Cif7Opts, context_association_lists::ContextAssociationLists,
    ecef_ephemeris::EcefEphemeris, formatted_gps::FormattedGps, gain::Gain, gps_ascii::GpsAscii,
};
use deku::prelude::*;
use fixed::types::extra::{U20, U7};
use fixed::{FixedI16, FixedI64, FixedU64};
use vita49_macros::{ack_field, cif_basic, cif_field, cif_fields, cif_radix, cif_radix_masked};

/// Base data structure for the CIF0 single-bit indicators.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cif0(u32);

impl Cif0 {
    cif_field!(context_field_changed, 31);
    cif_field!(reference_point_id, 30);
    cif_field!(bandwidth, 29);
    cif_field!(if_ref_freq, 28);
    cif_field!(rf_ref_freq, 27);
    cif_field!(rf_ref_freq_offset, 26);
    cif_field!(if_band_offset, 25);
    cif_field!(reference_level, 24);
    cif_field!(gain, 23);
    cif_field!(over_range_count, 22);
    cif_field!(sample_rate, 21);
    cif_field!(timestamp_adjustment, 20);
    cif_field!(timestamp_cal_time, 19);
    cif_field!(temperature, 18);
    cif_field!(device_id, 17);
    cif_field!(state_indicators, 16);
    cif_field!(signal_data_payload_format, 15);
    cif_field!(formatted_gps, 14);
    cif_field!(formatted_ins, 13);
    cif_field!(ecef_ephemeris, 12);
    cif_field!(relative_ephemeris, 11);
    cif_field!(ephemeris_ref_id, 10);
    cif_field!(gps_ascii, 9);
    cif_field!(context_association_lists, 8);
    cif_field!(field_attributes_enabled, 7);
    // Bits 4-6 are reserved
    cif_field!(cif3_enabled, 3);
    cif_field!(cif2_enabled, 2);
    cif_field!(cif1_enabled, 1);
    // Bit 0 is reserved

    /// Get the CIF field as a raw u32.
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Returns true if the whole CIF is empty.
    pub fn empty(&self) -> bool {
        self.0 == 0
    }
}

#[cif_fields(cif0)]
pub struct Cif0Fields {
    reference_point_id: u32,
    bandwidth: u64,
    if_ref_freq: i64,
    rf_ref_freq: u64,
    rf_ref_freq_offset: i64,
    if_band_offset: i64,
    reference_level: i32,
    gain: Gain,
    over_range_count: u32,
    sample_rate: u64,
    timestamp_adjustment: u64,
    timestamp_cal_time: u32,
    temperature: i32,
    device_id: DeviceId,
    state_indicators: u32,
    signal_data_payload_format: u64,
    formatted_gps: FormattedGps,
    formatted_ins: FormattedGps,
    ecef_ephemeris: EcefEphemeris,
    relative_ephemeris: EcefEphemeris,
    ephemeris_ref_id: u32,
    gps_ascii: GpsAscii,
    context_association_lists: ContextAssociationLists,
}

#[cif_fields(cif0)]
pub struct Cif0AckFields {
    reference_point_id: AckResponse,
    bandwidth: AckResponse,
    if_ref_freq: AckResponse,
    rf_ref_freq: AckResponse,
    rf_ref_freq_offset: AckResponse,
    if_band_offset: AckResponse,
    reference_level: AckResponse,
    gain: AckResponse,
    over_range_count: AckResponse,
    sample_rate: AckResponse,
    timestamp_adjustment: AckResponse,
    timestamp_cal_time: AckResponse,
    temperature: AckResponse,
    device_id: AckResponse,
    state_indicators: AckResponse,
    signal_data_payload_format: AckResponse,
    formatted_gps: AckResponse,
    formatted_ins: AckResponse,
    ecef_ephemeris: AckResponse,
    relative_ephemeris: AckResponse,
    ephemeris_ref_id: AckResponse,
    gps_ascii: AckResponse,
    context_association_lists: AckResponse,
}

/// Trait for common CIF0 manipulation methods. Used by Context and
/// Command packets.
#[rustfmt::skip]
pub trait Cif0Manipulators {
    /// Get a reference to the packet's CIF0 (indicators)
    fn cif0(&self) -> &Cif0;
    /// Get a mutable reference to the packet's CIF0 (indicators)
    fn cif0_mut(&mut self) -> &mut Cif0;
    /// Get a reference to the packet's CIF0 data fields
    fn cif0_fields(&self) -> &Cif0Fields;
    /// Get a mutable reference to the packet's CIF0 data fields
    fn cif0_fields_mut(&mut self) -> &mut Cif0Fields;

    cif_basic!(cif0, reference_point_id, reference_point_id, u32);
    cif_radix!(cif0, bandwidth, bandwidth_hz, f64, FixedU64::<U20>);
    cif_radix!(cif0, if_ref_freq, if_ref_freq_hz, f64, FixedI64::<U20>);
    cif_radix!(cif0, rf_ref_freq, rf_ref_freq_hz, f64, FixedU64::<U20>);
    cif_radix!(cif0, rf_ref_freq_offset, rf_ref_freq_offset_hz, f64, FixedI64::<U20>);
    cif_radix!(cif0, if_band_offset, if_band_offset_hz, f64, FixedI64::<U20>);
    cif_radix_masked!(cif0, reference_level, reference_level_db, f32, FixedI16::<U7>, i32, i16);
    cif_basic!(cif0, gain, gain, Gain);
    cif_basic!(cif0, over_range_count, over_range_count, u32);
    cif_radix!(cif0, sample_rate, sample_rate_sps, f64, FixedU64::<U20>);
    // TODO: add full support
    cif_basic!(cif0, timestamp_adjustment, timestamp_adjustment, u64);
    // TODO: add full support
    cif_basic!(cif0, timestamp_cal_time, timestamp_cal_time, u32);
    // TODO: add full support
    cif_basic!(cif0, temperature, temperature, i32);
    cif_basic!(cif0, device_id, device_id, DeviceId);
    // TODO: add full support
    cif_basic!(cif0, state_indicators, state_indicators, u32);
    // TODO: add full support
    cif_basic!(cif0, signal_data_payload_format, signal_data_payload_format, u64);
    cif_basic!(cif0, formatted_gps, formatted_gps, FormattedGps);
    cif_basic!(cif0, formatted_ins, formatted_ins, FormattedGps);
    cif_basic!(cif0, ecef_ephemeris, ecef_ephemeris, EcefEphemeris);
    cif_basic!(cif0, relative_ephemeris, relative_ephemeris, EcefEphemeris);
    cif_basic!(cif0, gps_ascii, gps_ascii, GpsAscii);
    cif_basic!(cif0, context_association_lists, context_association_lists, ContextAssociationLists);
}

/// Shared trait for manipulating CIF0 ACK fields.
pub trait Cif0AckManipulators {
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

    ack_field!(0, reference_point_id);
    ack_field!(0, bandwidth);
    ack_field!(0, if_ref_freq);
    ack_field!(0, rf_ref_freq);
    ack_field!(0, rf_ref_freq_offset);
    ack_field!(0, if_band_offset);
    ack_field!(0, reference_level);
    ack_field!(0, gain);
    ack_field!(0, over_range_count);
    ack_field!(0, sample_rate);
    ack_field!(0, timestamp_adjustment);
    ack_field!(0, timestamp_cal_time);
    ack_field!(0, temperature);
    ack_field!(0, device_id);
    ack_field!(0, state_indicators);
    ack_field!(0, signal_data_payload_format);
    ack_field!(0, formatted_gps);
    ack_field!(0, formatted_ins);
    ack_field!(0, ecef_ephemeris);
    ack_field!(0, relative_ephemeris);
    ack_field!(0, ephemeris_ref_id);
    ack_field!(0, gps_ascii);
    ack_field!(0, context_association_lists);
}

impl fmt::Display for Cif0 {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CIF0:")?;
        writeln!(f, "  Context field change indicator: {}", self.context_field_changed())?;
        writeln!(f, "  Reference point identifier: {}", self.reference_point_id())?;
        writeln!(f, "  Bandwidth: {}", self.bandwidth())?;
        writeln!(f, "  IF reference frequency: {}", self.if_ref_freq())?;
        writeln!(f, "  RF reference frequency: {}", self.rf_ref_freq())?;
        writeln!(f, "  RF reference frequency offset: {}", self.rf_ref_freq_offset())?;
        writeln!(f, "  IF band offset: {}", self.if_band_offset())?;
        writeln!(f, "  Reference level: {}", self.reference_level())?;
        writeln!(f, "  Gain: {}", self.gain())?;
        writeln!(f, "  Over-range count: {}", self.over_range_count())?;
        writeln!(f, "  Sample rate: {}", self.sample_rate())?;
        writeln!(f, "  Timestamp adjustment: {}", self.timestamp_adjustment())?;
        writeln!(f, "  Timestamp calibration time: {}", self.timestamp_cal_time())?;
        writeln!(f, "  Temperature: {}", self.temperature())?;
        writeln!(f, "  Device identifier: {}", self.device_id())?;
        writeln!(f, "  State/event indicators: {}", self.state_indicators())?;
        writeln!(f, "  Signal data format: {}", self.signal_data_payload_format())?;
        writeln!(f, "  Formatted GPS: {}", self.formatted_gps())?;
        writeln!(f, "  Formatted INS: {}", self.formatted_ins())?;
        writeln!(f, "  ECEF ephemeris: {}", self.ecef_ephemeris())?;
        writeln!(f, "  Relative ephemeris: {}", self.relative_ephemeris())?;
        writeln!(f, "  Ephemeris ref ID: {}", self.ephemeris_ref_id())?;
        writeln!(f, "  GPS ASCII: {}", self.gps_ascii())?;
        writeln!(f, "  Context association lists: {}", self.context_association_lists())?;
        writeln!(f, "  CIF7: {}", self.field_attributes_enabled())?;
        writeln!(f, "  CIF3: {}", self.cif3_enabled())?;
        writeln!(f, "  CIF2: {}", self.cif2_enabled())?;
        writeln!(f, "  CIF1: {}", self.cif1_enabled())?;
        Ok(())
    }
}
