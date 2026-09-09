// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Defines fields and methods related to CIF1 (ANSI/VITA-49.2-2017 9.1).
Fields here are compatible with VITA 49.2 and later.
*/

use core::fmt;

use crate::{
    ack::AckLevel, ack_response::AckResponse, cif0::Cif0, cif7::Cif7Opts, gain::Gain,
    spectrum::Spectrum, Cif0AckFields, Threshold,
};
use deku::prelude::*;
use fixed::{
    types::extra::{U20, U6, U7},
    FixedI16, FixedI32, FixedU64,
};
use vita49_macros::{
    ack_field, cif_basic, cif_field, cif_fields, cif_radix, cif_radix_masked, todo_cif_field,
};

/// Base data structure for the CIF1 single-bit indicators
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cif1(u32);

impl Cif1 {
    cif_field!(phase_offset, 31);
    cif_field!(polarization, 30);
    cif_field!(three_d_pointing_vector, 29);
    todo_cif_field!(three_d_pointing_vector_struct, 28, 1);
    cif_field!(spatial_scan_type, 27);
    cif_field!(spatial_ref_type, 26);
    cif_field!(beam_widths, 25);
    cif_field!(range, 24);
    // Bits 21 - 23 are reserved
    cif_field!(eb_over_no_and_ber, 20);
    cif_field!(threshold, 19);
    cif_field!(compression_point, 18);
    cif_field!(second_and_third_order_intercept_points, 17);
    cif_field!(snr_figure, 16);
    cif_field!(aux_freq, 15);
    cif_field!(aux_gain, 14);
    cif_field!(aux_bandwidth, 13);
    // Bit 12 is reserved
    todo_cif_field!(array_of_cifs, 11, 1);
    cif_field!(spectrum, 10);
    todo_cif_field!(sector_scan, 9, 1);
    // Bit 8 is reserved
    todo_cif_field!(index_list, 7, 1);
    cif_field!(discrete_io_32, 6);
    cif_field!(discrete_io_64, 5);
    cif_field!(health_status, 4);
    cif_field!(v49_spec_compliance, 3);
    cif_field!(version_and_build_code, 2);
    cif_field!(buffer_size, 1);
    // Bit 0 is reserved

    fn empty(&self) -> bool {
        self.0 == 0
    }
}

#[cif_fields(cif1)]
pub struct Cif1Fields {
    // TODO: add full support
    phase_offset: i32,
    // TODO: add full support
    polarization: i32,
    // TODO: add full support
    three_d_pointing_vector: i32,
    // TODO: add basic support
    three_d_pointing_vector_struct: u32,
    // TODO: add full support
    spatial_scan_type: u32,
    // TODO: add full support
    spatial_ref_type: u32,
    // TODO: add full support
    beam_widths: u32,
    range: i32,
    // TODO: add full support
    eb_over_no_and_ber: i32,
    threshold: Threshold,
    compression_point: i32,
    // TODO: add full support
    second_and_third_order_intercept_points: i32,
    // TODO: add full support
    snr_figure: i32,
    aux_freq: u64,
    aux_gain: Gain,
    aux_bandwidth: u64,
    // TODO: add basic support
    array_of_cifs: u32,
    spectrum: Spectrum,
    // TODO: add basic support
    sector_scan: u32,
    // TODO: add basic support
    index_list: u32,
    discrete_io_32: u32,
    discrete_io_64: u64,
    // TODO: add full support
    health_status: u32,
    // TODO: add full support
    v49_spec_compliance: u32,
    // TODO: add full support
    version_and_build_code: u32,
    // TODO: add full support
    buffer_size: u64,
}

#[cif_fields(cif1)]
pub struct Cif1AckFields {
    phase_offset: AckResponse,
    polarization: AckResponse,
    three_d_pointing_vector: AckResponse,
    three_d_pointing_vector_struct: AckResponse,
    spatial_scan_type: AckResponse,
    spatial_ref_type: AckResponse,
    beam_widths: AckResponse,
    range: AckResponse,
    eb_over_no_and_ber: AckResponse,
    threshold: AckResponse,
    compression_point: AckResponse,
    second_and_third_order_intercept_points: AckResponse,
    snr_figure: AckResponse,
    aux_freq: AckResponse,
    aux_gain: AckResponse,
    aux_bandwidth: AckResponse,
    array_of_cifs: AckResponse,
    spectrum: AckResponse,
    sector_scan: AckResponse,
    index_list: AckResponse,
    discrete_io_32: AckResponse,
    discrete_io_64: AckResponse,
    health_status: AckResponse,
    v49_spec_compliance: AckResponse,
    version_and_build_code: AckResponse,
    buffer_size: AckResponse,
}

/// Trait for common CIF1 manipulation methods. Used by Context and
/// Command packets.
#[rustfmt::skip]
pub trait Cif1Manipulators {
    /// Get a reference to the packet's CIF0 (indicators)
    fn cif0(&self) -> &Cif0;
    /// Get a mutable reference to the packet's CIF0 (indicators)
    fn cif0_mut(&mut self) -> &mut Cif0;
    /// Get a reference to the packet's CIF1 (indicators)
    fn cif1(&self) -> Option<&Cif1>;
    /// Get a mutable reference to the packet's CIF1 (indicators)
    fn cif1_mut(&mut self) -> &mut Option<Cif1>;
    /// Get a reference to the packet's CIF1 data fields
    fn cif1_fields(&self) -> Option<&Cif1Fields>;
    /// Get a mutable reference to the packet's CIF1 data fields
    fn cif1_fields_mut(&mut self) -> &mut Option<Cif1Fields>;

    cif_radix_masked!(cif1, phase_offset, phase_offset_radians, f32, FixedI16::<U7>, i32, i16);
    // TODO: add full support
    cif_basic!(cif1, polarization, polarization, i32);
    // TODO: add full support
    cif_basic!(cif1, three_d_pointing_vector, three_d_pointing_vector, i32);
    // TODO: add basic support
    cif_basic!(cif1, three_d_pointing_vector_struct, three_d_pointing_vector_struct, u32);
    // TODO: add full support
    cif_basic!(cif1, spatial_scan_type, spatial_scan_type, u32);
    // TODO: add full support
    cif_basic!(cif1, spatial_ref_type, spatial_ref_type, u32);
    // TODO: add full support
    cif_basic!(cif1, beam_widths, beam_widths, u32);
    cif_radix!(cif1, range, range_m, f32, FixedI32::<U6>);
    // TODO: add full support
    cif_basic!(cif1, eb_over_no_and_ber, eb_over_no_and_ber, i32);
    cif_basic!(cif1, threshold, threshold, Threshold);
    cif_radix_masked!(cif1, compression_point, compression_point_dbm, f32, FixedI16::<U7>, i32, i16);
    // TODO: add full support
    cif_basic!(cif1, second_and_third_order_intercept_points, second_and_third_order_intercept_points, i32);
    // TODO: add full support
    cif_basic!(cif1, snr_figure, snr_figure, i32);
    cif_radix!(cif1, aux_freq, aux_freq_hz, f64, FixedU64::<U20>);
    cif_basic!(cif1, aux_gain, aux_gain, Gain);
    cif_radix!(cif1, aux_bandwidth, aux_bandwidth_hz, f64, FixedU64::<U20>);
    // TODO: add basic support
    cif_basic!(cif1, array_of_cifs, array_of_cifs, u32);
    cif_basic!(cif1, spectrum, spectrum, Spectrum);
    // TODO: add basic support
    cif_basic!(cif1, sector_scan, sector_scan, u32);
    // TODO: add basic support
    cif_basic!(cif1, index_list, index_list, u32);
    cif_basic!(cif1, discrete_io_32, discrete_io_32, u32);
    cif_basic!(cif1, discrete_io_64, discrete_io_64, u64);
    // TODO: add full support
    cif_basic!(cif1, health_status, health_status, u32);
    // TODO: add full support
    cif_basic!(cif1, v49_spec_compliance, v49_spec_compliance, u32);
    // TODO: add full support
    cif_basic!(cif1, version_and_build_code, version_and_build_code, u32);
    // TODO: add full support
    cif_basic!(cif1, buffer_size, buffer_size, u64);
}

/// Shared trait for manipulating CIF1 ACK fields.
pub trait Cif1AckManipulators {
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

    /// Get a reference to the packet's WIF1 (indicators)
    fn wif1(&self) -> Option<&Cif1>;
    /// Get a mutable reference to the packet's WIF1 (indicators)
    fn wif1_mut(&mut self) -> &mut Option<Cif1>;
    /// Get a reference to the packet's WIF1 data fields
    fn wif1_fields(&self) -> Option<&Cif1AckFields>;
    /// Get a mutable reference to the packet's WIF1 data fields
    fn wif1_fields_mut(&mut self) -> &mut Option<Cif1AckFields>;

    /// Get a reference to the packet's EIF1 (indicators)
    fn eif1(&self) -> Option<&Cif1>;
    /// Get a mutable reference to the packet's EIF1 (indicators)
    fn eif1_mut(&mut self) -> &mut Option<Cif1>;
    /// Get a reference to the packet's EIF1 data fields
    fn eif1_fields(&self) -> Option<&Cif1AckFields>;
    /// Get a mutable reference to the packet's EIF1 data fields
    fn eif1_fields_mut(&mut self) -> &mut Option<Cif1AckFields>;

    ack_field!(1, phase_offset);
    ack_field!(1, polarization);
    ack_field!(1, three_d_pointing_vector);
    ack_field!(1, three_d_pointing_vector_struct);
    ack_field!(1, spatial_scan_type);
    ack_field!(1, spatial_ref_type);
    ack_field!(1, beam_widths);
    ack_field!(1, range);
    ack_field!(1, eb_over_no_and_ber);
    ack_field!(1, threshold);
    ack_field!(1, compression_point);
    ack_field!(1, second_and_third_order_intercept_points);
    ack_field!(1, snr_figure);
    ack_field!(1, aux_freq);
    ack_field!(1, aux_gain);
    ack_field!(1, aux_bandwidth);
    ack_field!(1, array_of_cifs);
    ack_field!(1, spectrum);
    ack_field!(1, sector_scan);
    ack_field!(1, index_list);
    ack_field!(1, discrete_io_32);
    ack_field!(1, discrete_io_64);
    ack_field!(1, health_status);
    ack_field!(1, v49_spec_compliance);
    ack_field!(1, version_and_build_code);
    ack_field!(1, buffer_size);
}

impl fmt::Display for Cif1 {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CIF1:")?;
        writeln!(f, "  Phase offset: {}", self.phase_offset())?;
        writeln!(f, "  Polarization: {}", self.polarization())?;
        writeln!(f, "  3D pointing vector: {}", self.three_d_pointing_vector())?;
        writeln!(f, "  3D pointing vector struct: {}", self.three_d_pointing_vector_struct())?;
        writeln!(f, "  Spatial scan type: {}", self.spatial_scan_type())?;
        writeln!(f, "  Spatial ref type: {}", self.spatial_ref_type())?;
        writeln!(f, "  Beam widths: {}", self.beam_widths())?;
        writeln!(f, "  Range: {}", self.range())?;
        writeln!(f, "  Eb/No BER: {}", self.eb_over_no_and_ber())?;
        writeln!(f, "  Threshold: {}", self.threshold())?;
        writeln!(f, "  Compression point: {}", self.compression_point())?;
        writeln!(f, "  2nd and 3rd order intercept points: {}", self.second_and_third_order_intercept_points())?;
        writeln!(f, "  SNR figure: {}", self.snr_figure())?;
        writeln!(f, "  Aux frequency: {}", self.aux_freq())?;
        writeln!(f, "  Aux gain: {}", self.aux_gain())?;
        writeln!(f, "  Aux bandwidth: {}", self.aux_bandwidth())?;
        writeln!(f, "  Array of CIFs: {}", self.array_of_cifs())?;
        writeln!(f, "  Spectrum: {}", self.spectrum())?;
        writeln!(f, "  Sector scan: {}", self.sector_scan())?;
        writeln!(f, "  Index list: {}", self.index_list())?;
        writeln!(f, "  Discrete I/O (32-bit): {}", self.discrete_io_32())?;
        writeln!(f, "  Discrete I/O (64-bit): {}", self.discrete_io_64())?;
        writeln!(f, "  Health status: {}", self.health_status())?;
        writeln!(f, "  V49 spec compliance: {}", self.v49_spec_compliance())?;
        writeln!(f, "  Version and build code: {}", self.version_and_build_code())?;
        writeln!(f, "  Buffer size: {}", self.buffer_size())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn setting_cif1_radix_field_does_not_enable_cif7() {
        let mut packet = Vrt::new_context_packet();
        let context = packet.payload_mut().context_mut().unwrap();

        // Setting a CIF1 radix field (e.g. aux_freq_hz) must NOT set field_attributes_enabled in CIF0.
        context.set_aux_freq_hz(Some(10e6));
        assert!(Cif0Manipulators::cif0(context).cif1_enabled());
        assert!(context.cif1().unwrap().aux_freq());
        assert!(
            !Cif0Manipulators::cif0(context).field_attributes_enabled(),
            "setting aux_freq_hz must not enable CIF7 (field attributes)"
        );
    }

    #[cfg(feature = "cif7")]
    #[test]
    fn setting_cif1_radix_attributes_enables_cif7() {
        let mut packet = Vrt::new_context_packet();
        let context = packet.payload_mut().context_mut().unwrap();

        // Setting CIF1 radix attributes SHOULD set field_attributes_enabled in CIF0.
        context.set_aux_freq_hz_attributes(Some(vec![10e6, 12e6]));
        assert!(Cif0Manipulators::cif0(context).cif1_enabled());
        assert!(Cif0Manipulators::cif0(context).field_attributes_enabled());
    }
}
