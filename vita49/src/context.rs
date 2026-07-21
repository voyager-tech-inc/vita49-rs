// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use core::fmt;

use deku::prelude::*;

use crate::cif0::{Cif0, Cif0Fields, Cif0Manipulators};
use crate::cif1::{Cif1, Cif1Fields, Cif1Manipulators};
use crate::cif2::{Cif2, Cif2Fields, Cif2Manipulators};
use crate::cif3::{Cif3, Cif3Fields, Cif3Manipulators};
use crate::cif7::{Cif7, Cif7Opts};
use crate::payload::Payload;

/// Context packet payload. Includes all CIFs and optional fields.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Context {
    /// CIF0 indicator fields.
    cif0: Cif0,
    /// CIF1 indicator fields.
    #[deku(cond = "cif0.cif1_enabled()")]
    cif1: Option<Cif1>,
    /// CIF2 indicator fields.
    #[deku(cond = "cif0.cif2_enabled()")]
    cif2: Option<Cif2>,
    /// CIF3 indicator fields.
    #[deku(cond = "cif0.cif3_enabled()")]
    cif3: Option<Cif3>,
    /// CIF7 indicator fields.
    #[deku(cond = "cif0.field_attributes_enabled()")]
    pub cif7: Option<Cif7>,

    /// CIF0 data fields.
    #[deku(ctx = "cif0, Cif7Opts::from(cif7.as_ref())")]
    cif0_fields: Cif0Fields,
    /// CIF1 data fields.
    #[deku(
        cond = "cif0.cif1_enabled()",
        ctx = "cif1.as_ref(), Cif7Opts::from(cif7.as_ref())"
    )]
    cif1_fields: Option<Cif1Fields>,
    /// CIF2 data fields.
    #[deku(
        cond = "cif0.cif2_enabled()",
        ctx = "cif2.as_ref(), Cif7Opts::from(cif7.as_ref())"
    )]
    cif2_fields: Option<Cif2Fields>,
    /// CIF3 data fields.
    #[deku(
        cond = "cif0.cif3_enabled()",
        ctx = "cif3.as_ref(), Cif7Opts::from(cif7.as_ref())"
    )]
    cif3_fields: Option<Cif3Fields>,
}

impl Context {
    /// Create a new context payload with no CIF bits or fields
    /// set.
    pub fn new() -> Context {
        Context::default()
    }

    /// Returns true if the context field change indicator is set, false if not.
    pub fn context_changed(&self) -> bool {
        self.cif0.context_field_changed()
    }

    /// Set the context field change indicator bit.
    pub fn set_context_changed(&mut self, changed: bool) {
        if changed {
            self.cif0.set_context_field_changed()
        } else {
            self.cif0.unset_context_field_changed()
        }
    }

    /// Returns the size of the context payload in 32-bit words.
    pub fn size_words(&self) -> u16 {
        // Start with 1 32-bit word for the CIF0 field
        let mut ret = 1 + self.cif0_fields.size_words();
        if let Some(f) = &self.cif1_fields {
            ret += 1 + f.size_words();
        }
        if let Some(f) = &self.cif2_fields {
            ret += 1 + f.size_words();
        }
        if let Some(f) = &self.cif3_fields {
            ret += 1 + f.size_words();
        }
        if self.cif0.field_attributes_enabled() {
            ret += 1;
        }
        ret
    }
}

impl TryFrom<Payload> for Context {
    type Error = Payload;

    fn try_from(value: Payload) -> Result<Self, Self::Error> {
        match value {
            Payload::Context(c) => Ok(c),
            a => Err(a),
        }
    }
}

impl Cif0Manipulators for Context {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif0_fields(&self) -> &Cif0Fields {
        &self.cif0_fields
    }
    fn cif0_fields_mut(&mut self) -> &mut Cif0Fields {
        &mut self.cif0_fields
    }
}

impl Cif1Manipulators for Context {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif1(&self) -> Option<&Cif1> {
        self.cif1.as_ref()
    }
    fn cif1_mut(&mut self) -> &mut Option<Cif1> {
        &mut self.cif1
    }
    fn cif1_fields(&self) -> Option<&Cif1Fields> {
        self.cif1_fields.as_ref()
    }
    fn cif1_fields_mut(&mut self) -> &mut Option<Cif1Fields> {
        &mut self.cif1_fields
    }
}

impl Cif2Manipulators for Context {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif2(&self) -> Option<&Cif2> {
        self.cif2.as_ref()
    }
    fn cif2_mut(&mut self) -> &mut Option<Cif2> {
        &mut self.cif2
    }
    fn cif2_fields(&self) -> Option<&Cif2Fields> {
        self.cif2_fields.as_ref()
    }
    fn cif2_fields_mut(&mut self) -> &mut Option<Cif2Fields> {
        &mut self.cif2_fields
    }
}

impl Cif3Manipulators for Context {
    fn cif0(&self) -> &Cif0 {
        &self.cif0
    }
    fn cif0_mut(&mut self) -> &mut Cif0 {
        &mut self.cif0
    }
    fn cif3(&self) -> Option<&Cif3> {
        self.cif3.as_ref()
    }
    fn cif3_mut(&mut self) -> &mut Option<Cif3> {
        &mut self.cif3
    }
    fn cif3_fields(&self) -> Option<&Cif3Fields> {
        self.cif3_fields.as_ref()
    }
    fn cif3_fields_mut(&mut self) -> &mut Option<Cif3Fields> {
        &mut self.cif3_fields
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cif0)?;
        if let Some(cif1) = self.cif1 {
            write!(f, "{cif1}")?;
        }
        if let Some(bw) = &self.bandwidth_hz() {
            writeln!(f, "Bandwidth: {bw} Hz")?;
        }
        if let Some(rf_freq) = &self.rf_ref_freq_hz() {
            writeln!(f, "RF reference frequency: {rf_freq} Hz")?;
        }
        if let Some(samp_rate) = &self.sample_rate_sps() {
            writeln!(f, "Sample rate: {samp_rate} sps")?;
        }
        if let Some(device_id) = &self.device_id() {
            write!(f, "{device_id}")?;
        }
        if let Some(spectrum) = self.spectrum() {
            write!(f, "{spectrum}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    #[test]
    fn read_context_internals() {
        use crate::prelude::*;
        let json = include_str!("../tests/context_packet.json5");
        let packet: Vrt = serde_json5::from_str(json).unwrap();
        assert_eq!(packet.header().packet_type(), PacketType::Context);
        assert!(packet.header().stream_id_included());
        assert!(!packet.header().class_id_included());
        assert!(packet.header().integer_timestamp_included());
        assert!(packet.header().fractional_timestamp_included());
        assert!(!packet.header().trailer_included());
        assert_eq!(packet.header().tsi(), Tsi::Utc);
        assert_eq!(packet.header().tsf(), Tsf::RealTimePs);
        assert_eq!(packet.stream_id(), Some(1));

        let context = packet.payload().context().unwrap();
        log::info!("\nParsed context packet:\n{}", context);
        assert!(!context.context_changed());
        assert!(!context.cif0.reference_point_id());
        assert!(context.cif0.bandwidth());
        assert_eq!(context.bandwidth_hz(), Some(6e6));
        assert_eq!(context.rf_ref_freq_hz(), Some(100e6));
        assert_eq!(context.sample_rate_sps(), Some(8e6));
        assert!(context.cif0.cif1_enabled());
        assert!(context.cif1.is_some());
        assert!(context.cif1.clone().unwrap().spectrum());
        assert_eq!(context.spectrum().unwrap().spectrum_type_as_u32(), 0x101);
        assert_eq!(context.spectrum().unwrap().num_transform_points(), 1280);
        assert_eq!(context.spectrum().unwrap().f1_index(), -640);
    }

    #[test]
    fn negative_reference_level_does_not_corrupt_upper_bits() {
        // Regression (cif_radix_masked): negative fixed-point to_bits() was cast
        // directly to i32, sign-extending into reserved bits 16-31.
        use crate::prelude::*;
        use approx::assert_relative_eq;
        let mut packet = Vrt::new_context_packet();
        let context = packet.payload_mut().context_mut().unwrap();
        context.set_reference_level_db(Some(-30.0));
        let raw = context.cif0_fields().reference_level.unwrap();
        assert_eq!(
            raw >> 16,
            0,
            "upper 16 bits of reference_level must be zero"
        );
        assert_relative_eq!(
            context.reference_level_db().unwrap(),
            -30.0_f32,
            max_relative = 0.1
        );
    }
}
