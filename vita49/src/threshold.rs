// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the threshold field described in
(ANSI/VITA-49.2-2017 section 9.5.13).
*/

use deku::prelude::*;
use fixed::{types::extra::U7, FixedI16};
use std::fmt;

/// Base threshold data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Threshold(i32);

impl Threshold {
    /// Create a new `Threshold` object given stage 1 and 2 in dB.
    pub fn new(stage_1_threshold_db: f32, stage_2_threshold_db: f32) -> Threshold {
        let s1 = FixedI16::<U7>::from_num(stage_1_threshold_db).to_bits() as u16 as i32;
        let s2 = FixedI16::<U7>::from_num(stage_2_threshold_db).to_bits() as u16 as i32;
        Threshold((s2 << 16) | s1)
    }

    /// Gets the size of the threshold structure in 32-bit words.
    pub fn size_words(&self) -> u16 {
        (std::mem::size_of_val(&self.0) / std::mem::size_of::<u32>()) as u16
    }

    /// Gets stage 1 threshold (dB)
    pub fn stage_1_threshold_db(&self) -> f32 {
        let s1 = (self.0 & 0xFFFF) as i16;
        FixedI16::<U7>::from_bits(s1).to_num()
    }

    /// Sets stage 1 threshold (dB)
    pub fn set_stage_1_threshold_db(&mut self, stage_1_threshold_db: f32) {
        let s1 = FixedI16::<U7>::from_num(stage_1_threshold_db).to_bits() as u16 as i32;
        self.0 = (self.0 & (0xFFFF_0000u32 as i32)) | s1
    }

    /// Gets stage 2 threshold (dB)
    pub fn stage_2_threshold_db(&self) -> f32 {
        let s2 = ((self.0 >> 16) & 0xFFFF) as i16;
        FixedI16::<U7>::from_bits(s2).to_num()
    }

    /// Sets stage 2 threshold (dB)
    pub fn set_stage_2_threshold_db(&mut self, stage_2_threshold_db: f32) {
        let s2 = FixedI16::<U7>::from_num(stage_2_threshold_db).to_bits() as i32;
        self.0 = (self.0 & 0x0000_FFFF) | (s2 << 16)
    }
}

impl fmt::Display for Threshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "Stage 1: {} dB, Stage 2: {} dB",
            self.stage_1_threshold_db(),
            self.stage_2_threshold_db()
        )
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::Threshold;

    #[test]
    fn manipulate_threshold() {
        let _ = env_logger::builder().is_test(true).try_init();
        use crate::prelude::*;
        let mut packet = Vrt::new_context_packet();
        let context = packet.payload_mut().context_mut().unwrap();
        let mut s1: f32 = 25.2;
        let mut s2: f32 = 0.23;
        let mut t = Threshold::new(s1, s2);
        context.set_threshold(Some(t));
        assert_relative_eq!(
            context.threshold().unwrap().stage_1_threshold_db(),
            s1,
            max_relative = 0.1
        );
        assert_relative_eq!(
            context.threshold().unwrap().stage_2_threshold_db(),
            s2,
            max_relative = 0.1
        );
        s1 = -20.5;
        s2 = -11.1;
        t.set_stage_1_threshold_db(s1);
        t.set_stage_2_threshold_db(s2);
        context.set_threshold(Some(t));
        assert_relative_eq!(
            context.threshold().unwrap().stage_1_threshold_db(),
            s1,
            max_relative = 0.1
        );
        assert_relative_eq!(
            context.threshold().unwrap().stage_2_threshold_db(),
            s2,
            max_relative = 0.1
        );
    }

    #[test]
    fn negative_stage1_does_not_clobber_stage2() {
        // Regression: negative s1 was sign-extended to i32 before the OR,
        // setting bits 16-31 and overwriting whatever s2 had placed there.
        let s1: f32 = -20.5;
        let s2: f32 = 3.0;
        let t = Threshold::new(s1, s2);
        assert_relative_eq!(t.stage_1_threshold_db(), s1, max_relative = 0.1);
        assert_relative_eq!(t.stage_2_threshold_db(), s2, max_relative = 0.1);

        let mut t2 = Threshold::new(0.0, s2);
        t2.set_stage_1_threshold_db(s1);
        assert_relative_eq!(t2.stage_1_threshold_db(), s1, max_relative = 0.1);
        assert_relative_eq!(t2.stage_2_threshold_db(), s2, max_relative = 0.1);
    }
}
