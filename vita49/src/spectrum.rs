// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0
/*!
Data structures and methods related to the spectral metadata field
(ANSI/VITA-49.2-2017 section 9.6.1).
*/

use core::fmt;
use std::convert::From;

use deku::prelude::*;
use fixed::{
    types::extra::{U12, U20},
    FixedI32, FixedI64,
};

use crate::VitaError;

/// Base spectrum field data structure.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spectrum {
    /// Spectrum type
    spectrum_type: u32,
    /// Window type
    window_type: u32,
    /// The size of the mathematical transform used to create the spectral
    /// data, i.e., FFT size
    num_transform_points: u32,
    /// In certain circumstances, particularly when using decimating algorithms,
    /// the # points in the window may be different than the # points in the
    /// spectrum transform.
    num_window_points: u32,
    /// Resolution of the spectral points, equivalent to a transform bin size
    /// or bandwidth, using the existing VITA49 bandwidth field definition for
    /// its format.
    resolution: i64,
    /// The overall bandwidth given by the spectral data; for a DFT it would
    /// describe the spectral extent (1st point to last point) of the data.
    /// This uses the existing bandwidth field definition for its format.
    span: i64,
    /// Describes the extent of averaging or smoothing applied to the data.
    num_averages: u32,
    /// Provides the mathematical coefficient when nonlinear (such as exponential)
    /// averaging is used. It can be used as a coefficient in a simple IIR filter
    /// implementation as well.
    weighting_factor: i32,
    /// Left-side index of subset of spectral data. Integer index number (not
    /// a frequency).
    f1_index: i32,
    /// Right-side index of subset of spectral data. Integer index number (not
    /// a frequency).
    f2_index: i32,
    /// Describes the amount of overlap in successive spectral transforms,
    /// in one of 3 ways: time, percent, # samples.
    window_time_delta: WindowTimeDelta,
}

macro_rules! size_of_fields {
    ($self:expr, $($field:ident),*) => {{
        let mut acc = 0;
        $(acc += (std::mem::size_of_val(&$self.$field) / std::mem::size_of::<u32>()) as u16;)*
        acc
    }}
}

/// Type of spectral data being presented.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpectrumType {
    /// Default "no setting".
    Default = 0,
    /// Log power (dB).
    LogPowerDb = 1,
    /// Cartesian (I, Q).
    Cartesian = 2,
    /// Polar (magnitued, phase).
    Polar = 3,
    /// Magnitude.
    Magnitude = 4,
    /// Reserved for future expansion.
    Reserved,
    /// User defined type.
    UserDefined(u8),
}

impl From<u8> for SpectrumType {
    fn from(value: u8) -> Self {
        match value {
            0 => SpectrumType::Default,
            1 => SpectrumType::LogPowerDb,
            2 => SpectrumType::Cartesian,
            3 => SpectrumType::Polar,
            4 => SpectrumType::Magnitude,
            5..=127 => SpectrumType::Reserved,
            128..=255 => SpectrumType::UserDefined(value),
        }
    }
}

impl From<SpectrumType> for u8 {
    fn from(value: SpectrumType) -> Self {
        match value {
            SpectrumType::Default => 0,
            SpectrumType::LogPowerDb => 1,
            SpectrumType::Cartesian => 2,
            SpectrumType::Polar => 3,
            SpectrumType::Magnitude => 4,
            SpectrumType::UserDefined(v) => v,
            SpectrumType::Reserved => panic!("can't convert reserved variant"),
        }
    }
}

/// Type of averaging being performed (ANSI/VITA-49.2-2017 section 9.6.1.1.2).
///
/// This is a bit-field where multiple averaging types can be selected simultaneously
/// (Permission 9.6.1.1.2-1). Bit 5 (`smoothing`) modifies whether averaging is
/// performed within a Sample Frame rather than across Sample Frames (Rule 9.6.1.1.2-9).
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AveragingType(u8);

impl AveragingType {
    /// Bit 0: linear averaging (Rule 9.6.1.1.2-3).
    pub fn linear(&self) -> bool {
        self.0 & (1 << 0) != 0
    }
    /// Set linear averaging.
    pub fn set_linear(&mut self) {
        self.0 |= 1 << 0;
    }
    /// Unset linear averaging.
    pub fn unset_linear(&mut self) {
        self.0 &= !(1 << 0);
    }

    /// Bit 1: peak hold (Rule 9.6.1.1.2-4).
    pub fn peak_hold(&self) -> bool {
        self.0 & (1 << 1) != 0
    }
    /// Set peak hold.
    pub fn set_peak_hold(&mut self) {
        self.0 |= 1 << 1;
    }
    /// Unset peak hold.
    pub fn unset_peak_hold(&mut self) {
        self.0 &= !(1 << 1);
    }

    /// Bit 2: min hold (Rule 9.6.1.1.2-5).
    pub fn min_hold(&self) -> bool {
        self.0 & (1 << 2) != 0
    }
    /// Set min hold.
    pub fn set_min_hold(&mut self) {
        self.0 |= 1 << 2;
    }
    /// Unset min hold.
    pub fn unset_min_hold(&mut self) {
        self.0 &= !(1 << 2);
    }

    /// Bit 3: exponential averaging (Rule 9.6.1.1.2-6).
    pub fn exponential(&self) -> bool {
        self.0 & (1 << 3) != 0
    }
    /// Set exponential averaging.
    pub fn set_exponential(&mut self) {
        self.0 |= 1 << 3;
    }
    /// Unset exponential averaging.
    pub fn unset_exponential(&mut self) {
        self.0 &= !(1 << 3);
    }

    /// Bit 4: median averaging (Rule 9.6.1.1.2-7).
    pub fn median(&self) -> bool {
        self.0 & (1 << 4) != 0
    }
    /// Set median averaging.
    pub fn set_median(&mut self) {
        self.0 |= 1 << 4;
    }
    /// Unset median averaging.
    pub fn unset_median(&mut self) {
        self.0 &= !(1 << 4);
    }

    /// Bit 5: smoothing within the Sample Frame (Rule 9.6.1.1.2-9).
    ///
    /// Per Rule 9.6.1.1.2-9, indicates that averaging is performed on data
    /// within a Sample Frame, as opposed to across multiple Sample Frames.
    pub fn smoothing(&self) -> bool {
        self.0 & (1 << 5) != 0
    }
    /// Set smoothing within the Sample Frame.
    pub fn set_smoothing(&mut self) {
        self.0 |= 1 << 5;
    }
    /// Unset smoothing within the Sample Frame.
    pub fn unset_smoothing(&mut self) {
        self.0 &= !(1 << 5);
    }

    /// Returns the raw u8 bit-field value.
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Returns true if no averaging type is selected.
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }
}

impl From<u8> for AveragingType {
    fn from(value: u8) -> Self {
        AveragingType(value)
    }
}

impl From<AveragingType> for u8 {
    fn from(value: AveragingType) -> Self {
        value.0
    }
}

/// Interpretation options for the window time delta field.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowTimeDeltaInterpretation {
    /// Overlap is not controlled.
    OverlapNotControlled = 0,
    /// Percent overlap.
    PercentOverlap = 1,
    /// Samples.
    Samples = 2,
    /// Time.
    Time = 3,
    /// Reserved for future expansion.
    Reserved,
}

impl From<u8> for WindowTimeDeltaInterpretation {
    fn from(value: u8) -> Self {
        match value {
            0 => WindowTimeDeltaInterpretation::OverlapNotControlled,
            1 => WindowTimeDeltaInterpretation::PercentOverlap,
            2 => WindowTimeDeltaInterpretation::Samples,
            3 => WindowTimeDeltaInterpretation::Time,
            _ => WindowTimeDeltaInterpretation::Reserved,
        }
    }
}

impl From<WindowTimeDeltaInterpretation> for u8 {
    fn from(value: WindowTimeDeltaInterpretation) -> Self {
        match value {
            WindowTimeDeltaInterpretation::OverlapNotControlled => 0,
            WindowTimeDeltaInterpretation::PercentOverlap => 1,
            WindowTimeDeltaInterpretation::Samples => 2,
            WindowTimeDeltaInterpretation::Time => 3,
            WindowTimeDeltaInterpretation::Reserved => panic!("can't convert reserved variant"),
        }
    }
}

/// Window type enumeration.
///
/// Some variants include an alpha coefficient as
/// a 3-digit suffix (e.g. `Hanning100` is a Hanning window
/// with a 1.00 alpha coefficient).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowType {
    /// Rectangle windowing.
    Rectangle = 0,
    /// Triangle windowing.
    Triangle = 1,
    /// Hanning windowing (1.00 "alpha").
    Hanning100 = 2,
    /// Hanning windowing (2.00 "alpha").
    Hanning200 = 3,
    /// Hanning windowing (3.00 "alpha").
    Hanning300 = 4,
    /// Hanning windowing (4.00 "alpha").
    Hanning400 = 5,
    /// Hamming windowing.
    Hamming = 6,
    /// Riesz windowing.
    Riesz = 7,
    /// Riemann windowing.
    Riemann = 8,
    /// De La Vallepoussin windowing.
    DeLaVallepoussin = 9,
    /// Tukey windowing (0.25 "alpha").
    Tukey025 = 10,
    /// Tukey windowing (0.50 "alpha").
    Tukey050 = 11,
    /// Tukey windowing (0.75 "alpha").
    Tukey075 = 12,
    /// Bohman windowing.
    Bohman = 13,
    /// Poisson windowing (2.00 "alpha").
    Poisson200 = 14,
    /// Poisson windowing (3.00 "alpha").
    Poisson300 = 15,
    /// Poisson windowing (4.00 "alpha").
    Poisson400 = 16,
    /// Hanning-Poisson windowing (0.50 "alpha").
    HanningPoisson050 = 17,
    /// Hanning-Poisson windowing (1.00 "alpha").
    HanningPoisson100 = 18,
    /// Hanning-Poisson windowing (2.00 "alpha").
    HanningPoisson200 = 19,
    /// Cauchy windowing (3.00 "alpha").
    Cauchy300 = 20,
    /// Cauchy windowing (4.00 "alpha").
    Cauchy400 = 21,
    /// Cauchy windowing (5.00 "alpha").
    Cauchy500 = 22,
    /// Gaussian windowing (2.50 "alpha").
    Gaussian250 = 23,
    /// Gaussian windowing (3.00 "alpha").
    Gaussian300 = 24,
    /// Gaussian windowing (3.50 "alpha").
    Gaussian350 = 25,
    /// Dolph-Chebyshiev windowing (2.50 "alpha").
    DolphChebyshiev250 = 26,
    /// Dolph-Chebyshiev windowing (3.00 "alpha").
    DolphChebyshiev300 = 27,
    /// Dolph-Chebyshiev windowing (3.50 "alpha").
    DolphChebyshiev350 = 28,
    /// Dolph-Chebyshiev windowing (4.00 "alpha").
    DolphChebyshiev400 = 29,
    /// Kaiser-Bessel windowing (2.00 "alpha").
    KaiserBessel200 = 30,
    /// Kaiser-Bessel windowing (2.50 "alpha").
    KaiserBessel250 = 31,
    /// Kaiser-Bessel windowing (3.00 "alpha").
    KaiserBessel300 = 32,
    /// Kaiser-Bessel windowing (3.50 "alpha").
    KaiserBessel350 = 33,
    /// Barcilon-Temes windowing (3.00 "alpha").
    BarcilonTemes300 = 34,
    /// Barcilon-Temes windowing (3.50 "alpha").
    BarcilonTemes350 = 35,
    /// Barcilon-Temes windowing (4.00 "alpha").
    BarcilonTemes400 = 36,
    /// Exact Blackman windowing.
    ExactBlackman = 37,
    /// Blackman windowing.
    Blackman = 38,
    /// Blackman-Harris windowing (minimum 3-sample).
    BlackmanHarrisMin3Sample = 39,
    /// Blackman-Harris windowing (minimum 4-sample).
    BlackmanHarrisMin4Sample = 40,
    /// Blackman-Harris windowing (61 dB, 3-sample).
    BlackmanHarris61Db3Sample = 41,
    /// Blackman-Harris windowing (74 dB, 4-sample).
    BlackmanHarris74Db4Sample = 42,
    /// Kaiser-Bessel windowing (4-sample, 3.00 "alpha").
    KaiserBessel4Sample300 = 43,
    /// Reserved for future expansion.
    Reserved,
    /// User-defined windowing scheme.
    Other(u8),
}

impl From<u8> for WindowType {
    fn from(value: u8) -> Self {
        match value {
            0 => WindowType::Rectangle,
            1 => WindowType::Triangle,
            2 => WindowType::Hanning100,
            3 => WindowType::Hanning200,
            4 => WindowType::Hanning300,
            5 => WindowType::Hanning400,
            6 => WindowType::Hamming,
            7 => WindowType::Riesz,
            8 => WindowType::Riemann,
            9 => WindowType::DeLaVallepoussin,
            10 => WindowType::Tukey025,
            11 => WindowType::Tukey050,
            12 => WindowType::Tukey075,
            13 => WindowType::Bohman,
            14 => WindowType::Poisson200,
            15 => WindowType::Poisson300,
            16 => WindowType::Poisson400,
            17 => WindowType::HanningPoisson050,
            18 => WindowType::HanningPoisson100,
            19 => WindowType::HanningPoisson200,
            20 => WindowType::Cauchy300,
            21 => WindowType::Cauchy400,
            22 => WindowType::Cauchy500,
            23 => WindowType::Gaussian250,
            24 => WindowType::Gaussian300,
            25 => WindowType::Gaussian350,
            26 => WindowType::DolphChebyshiev250,
            27 => WindowType::DolphChebyshiev300,
            28 => WindowType::DolphChebyshiev350,
            29 => WindowType::DolphChebyshiev400,
            30 => WindowType::KaiserBessel200,
            31 => WindowType::KaiserBessel250,
            32 => WindowType::KaiserBessel300,
            33 => WindowType::KaiserBessel350,
            34 => WindowType::BarcilonTemes300,
            35 => WindowType::BarcilonTemes350,
            36 => WindowType::BarcilonTemes400,
            37 => WindowType::ExactBlackman,
            38 => WindowType::Blackman,
            39 => WindowType::BlackmanHarrisMin3Sample,
            40 => WindowType::BlackmanHarrisMin4Sample,
            41 => WindowType::BlackmanHarris61Db3Sample,
            42 => WindowType::BlackmanHarris74Db4Sample,
            43 => WindowType::KaiserBessel4Sample300,
            44..=99 => WindowType::Reserved,
            100..=255 => WindowType::Other(value),
        }
    }
}

impl From<WindowType> for u8 {
    fn from(value: WindowType) -> Self {
        match value {
            WindowType::Rectangle => 0,
            WindowType::Triangle => 1,
            WindowType::Hanning100 => 2,
            WindowType::Hanning200 => 3,
            WindowType::Hanning300 => 4,
            WindowType::Hanning400 => 5,
            WindowType::Hamming => 6,
            WindowType::Riesz => 7,
            WindowType::Riemann => 8,
            WindowType::DeLaVallepoussin => 9,
            WindowType::Tukey025 => 10,
            WindowType::Tukey050 => 11,
            WindowType::Tukey075 => 12,
            WindowType::Bohman => 13,
            WindowType::Poisson200 => 14,
            WindowType::Poisson300 => 15,
            WindowType::Poisson400 => 16,
            WindowType::HanningPoisson050 => 17,
            WindowType::HanningPoisson100 => 18,
            WindowType::HanningPoisson200 => 19,
            WindowType::Cauchy300 => 20,
            WindowType::Cauchy400 => 21,
            WindowType::Cauchy500 => 22,
            WindowType::Gaussian250 => 23,
            WindowType::Gaussian300 => 24,
            WindowType::Gaussian350 => 25,
            WindowType::DolphChebyshiev250 => 26,
            WindowType::DolphChebyshiev300 => 27,
            WindowType::DolphChebyshiev350 => 28,
            WindowType::DolphChebyshiev400 => 29,
            WindowType::KaiserBessel200 => 30,
            WindowType::KaiserBessel250 => 31,
            WindowType::KaiserBessel300 => 32,
            WindowType::KaiserBessel350 => 33,
            WindowType::BarcilonTemes300 => 34,
            WindowType::BarcilonTemes350 => 35,
            WindowType::BarcilonTemes400 => 36,
            WindowType::ExactBlackman => 37,
            WindowType::Blackman => 38,
            WindowType::BlackmanHarrisMin3Sample => 39,
            WindowType::BlackmanHarrisMin4Sample => 40,
            WindowType::BlackmanHarris61Db3Sample => 41,
            WindowType::BlackmanHarris74Db4Sample => 42,
            WindowType::KaiserBessel4Sample300 => 43,
            WindowType::Other(v) => v,
            WindowType::Reserved => panic!("can't convert reserved variant"),
        }
    }
}

/// Window time delta structure.
///
/// Provides accessor methods that help handle the different
/// possible formats this field can have.
#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, DekuRead, DekuWrite,
)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WindowTimeDelta(u32);

impl WindowTimeDelta {
    /// Create a new window time-delta from some timestamp (nanoseconds).
    pub fn from_time_ns(time_ns: u32) -> WindowTimeDelta {
        WindowTimeDelta(time_ns)
    }

    /// Create a new window time-delta from some sample counter.
    pub fn from_samples(samples: u32) -> WindowTimeDelta {
        WindowTimeDelta(samples)
    }

    /// Create a new window time-delta from some percent overlap.
    pub fn from_percent_overlap(percent_overlap: f32) -> WindowTimeDelta {
        let mut ret = WindowTimeDelta::default();
        ret.set_percent_overlap(percent_overlap);
        ret
    }

    /// Get the window time-delta as nanoseconds (resolved to the nearest whole sample).
    pub fn as_time_ns(&self) -> u32 {
        self.0
    }

    /// Set the window time-delta as nanoseconds (resolved to the nearest whole sample).
    pub fn set_time_ns(&mut self, time_ns: u32) {
        self.0 = time_ns;
    }

    /// Get the window time-delta as raw samples.
    pub fn as_samples(&self) -> u32 {
        self.0
    }

    /// Get the window time-delta as raw samples.
    pub fn set_samples(&mut self, samples: u32) {
        self.0 = samples;
    }

    /// Get the window time-delta as percent overlap (resolved to the nearest whole sample).
    pub fn as_percent_overlap(&self) -> f32 {
        FixedI32::<U12>::from_bits(self.0 as i32).to_num()
    }

    /// Set the window time-delta as percent overlap (resolved to the nearest whole sample).
    pub fn set_percent_overlap(&mut self, percent_overlap: f32) {
        self.0 = FixedI32::<U12>::from_num(percent_overlap).to_bits() as u32;
    }
}

impl Spectrum {
    /// Generate a new spectrum object with default fields.
    pub fn new() -> Spectrum {
        Spectrum::default()
    }

    /// Gets the spectrum type
    pub fn spectrum_type(&self) -> SpectrumType {
        SpectrumType::from((self.spectrum_type & 0xFF) as u8)
    }

    /// Sets the spectrum type.
    ///
    /// # Errors
    /// This function can result in an error in two cases:
    /// 1. When a [`SpectrumType::UserDefined`] variant is passed, the value must
    ///    be between 128 and 255. If it's not, an error will be returned.
    /// 2. You may not set the spectrum type to the [`SpectrumType::Reserved`] variant.
    pub fn set_spectrum_type(&mut self, spectrum_type: SpectrumType) -> Result<(), VitaError> {
        match spectrum_type {
            SpectrumType::UserDefined(v) => {
                if v < 128 {
                    return Err(VitaError::OutOfRange);
                }
                self.spectrum_type = (self.spectrum_type & !(0xFF)) | (v as u32);
            }
            SpectrumType::Reserved => return Err(VitaError::ReservedField),
            _ => {
                self.spectrum_type =
                    (self.spectrum_type & !(0xFF)) | (u8::from(spectrum_type) as u32)
            }
        }
        Ok(())
    }

    /// Gets the averaging type.
    pub fn averaging_type(&self) -> AveragingType {
        AveragingType::from(((self.spectrum_type >> 8) & 0xFF) as u8)
    }

    /// Sets the averaging type.
    ///
    /// # Errors
    /// Bits 6 and 7 are reserved in VITA 49.2. If any reserved bits are set,
    /// returns [`VitaError::ReservedField`].
    pub fn set_averaging_type(&mut self, averaging_type: AveragingType) -> Result<(), VitaError> {
        if (averaging_type.as_u8() & 0b1100_0000) != 0 {
            return Err(VitaError::ReservedField);
        }
        let v = averaging_type.as_u8() as u32;
        self.spectrum_type = (self.spectrum_type & !(0xFF << 8)) | (v << 8);
        Ok(())
    }

    /// Gets the window time-delta interpretation.
    pub fn window_time_delta_interpretation(&self) -> WindowTimeDeltaInterpretation {
        WindowTimeDeltaInterpretation::from(((self.spectrum_type >> 16) & 0b1111) as u8)
    }

    /// Sets the window time-delta interpretation.
    ///
    /// # Errors
    /// You may not set this field to the [`WindowTimeDeltaInterpretation::Reserved`] variant.
    pub fn set_window_time_delta_interpretation(
        &mut self,
        window_time_delta_interpretation: WindowTimeDeltaInterpretation,
    ) -> Result<(), VitaError> {
        match window_time_delta_interpretation {
            WindowTimeDeltaInterpretation::Reserved => return Err(VitaError::ReservedField),
            _ => {
                let v = u8::from(window_time_delta_interpretation) as u32;
                self.spectrum_type = (self.spectrum_type & !(0b1111 << 16)) | (v << 16)
            }
        }
        Ok(())
    }

    /// Get the raw spectrum type field.
    pub fn spectrum_type_as_u32(&self) -> u32 {
        self.spectrum_type
    }

    /// Get the window type field.
    pub fn window_type(&self) -> WindowType {
        WindowType::from((self.window_type & 0xFF) as u8)
    }

    /// Set the window type field.
    ///
    // # Errors
    /// You may not set this field to the [`WindowType::Reserved`] variant.
    pub fn set_window_type(&mut self, window_type: WindowType) -> Result<(), VitaError> {
        if matches!(window_type, WindowType::Reserved) {
            return Err(VitaError::ReservedField);
        }
        self.window_type = u8::from(window_type) as u32;
        Ok(())
    }

    /// Get the number of transform points.
    pub fn num_transform_points(&self) -> u32 {
        self.num_transform_points
    }

    /// Set the number of transform points.
    pub fn set_num_transform_points(&mut self, num_transform_points: u32) {
        self.num_transform_points = num_transform_points;
    }

    /// Get the number of window points.
    pub fn num_window_points(&self) -> u32 {
        self.num_window_points
    }

    /// Set the number of window points.
    pub fn set_num_window_points(&mut self, num_window_points: u32) {
        self.num_window_points = num_window_points;
    }

    /// Get the spectral resolution (Hz).
    pub fn resolution_hz(&self) -> f64 {
        FixedI64::<U20>::from_bits(self.resolution).to_num()
    }

    /// Set the spectral resolution (Hz).
    pub fn set_resolution_hz(&mut self, resolution_hz: f64) {
        self.resolution = FixedI64::<U20>::from_num(resolution_hz).to_bits();
    }

    /// Get the spectral span (Hz).
    pub fn span_hz(&self) -> f64 {
        FixedI64::<U20>::from_bits(self.span).to_num()
    }

    /// Set the spectral span (Hz).
    pub fn set_span_hz(&mut self, span_hz: f64) {
        self.span = FixedI64::<U20>::from_num(span_hz).to_bits();
    }

    /// Get the number of averages.
    pub fn num_averages(&self) -> u32 {
        self.num_averages
    }

    /// Set the number of averages.
    pub fn set_num_averages(&mut self, num_averages: u32) {
        self.num_averages = num_averages;
    }

    /// Get the weighting factor.
    pub fn weighting_factor(&self) -> i32 {
        self.weighting_factor
    }

    /// Set the weighting factor.
    pub fn set_weighting_factor(&mut self, weighting_factor: i32) {
        self.weighting_factor = weighting_factor;
    }

    /// Get the F1 index.
    pub fn f1_index(&self) -> i32 {
        self.f1_index
    }

    /// Set the F1 index.
    pub fn set_f1_index(&mut self, f1_index: i32) {
        self.f1_index = f1_index;
    }

    /// Get the F2 index.
    pub fn f2_index(&self) -> i32 {
        self.f2_index
    }

    /// Set the F2 index.
    pub fn set_f2_index(&mut self, f2_index: i32) {
        self.f2_index = f2_index;
    }

    /// Get the window time delta.
    pub fn window_time_delta(&self) -> WindowTimeDelta {
        self.window_time_delta
    }

    /// Set the window time delta.
    pub fn set_window_time_delta(&mut self, window_time_delta: WindowTimeDelta) {
        self.window_time_delta = window_time_delta;
    }

    /// Gets the size of the spectral field in 32-bit words.
    pub fn size_words(&self) -> u16 {
        size_of_fields!(
            self,
            spectrum_type,
            window_type,
            num_transform_points,
            num_window_points,
            resolution,
            span,
            num_averages,
            weighting_factor,
            f1_index,
            f2_index,
            window_time_delta
        )
    }
}

impl fmt::Display for Spectrum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Spectrum:")?;
        writeln!(f, "  Spectrum type: {:x}", self.spectrum_type)?;
        writeln!(f, "  Window type: {:x}", self.window_type)?;
        writeln!(f, "  Num transform points: {}", self.num_transform_points())?;
        writeln!(f, "  Num window points: {}", self.num_window_points())?;
        writeln!(f, "  Resolution: {} Hz", self.resolution_hz())?;
        writeln!(f, "  Span: {} Hz", self.span_hz())?;
        writeln!(f, "  Num averages: {}", self.num_averages())?;
        writeln!(f, "  Weighting factor: {}", self.weighting_factor())?;
        writeln!(f, "  F1 index: {}", self.f1_index())?;
        writeln!(f, "  F2 index: {}", self.f2_index())?;
        match self.window_time_delta_interpretation() {
            WindowTimeDeltaInterpretation::PercentOverlap => {
                writeln!(
                    f,
                    "  Window time-delta: {}%",
                    self.window_time_delta.as_percent_overlap()
                )?;
            }
            WindowTimeDeltaInterpretation::Samples => {
                writeln!(
                    f,
                    "  Window time-delta: {} samples",
                    self.window_time_delta.as_samples()
                )?;
            }
            WindowTimeDeltaInterpretation::Time => {
                writeln!(
                    f,
                    "  Window time-delta: {} ns",
                    self.window_time_delta.as_time_ns()
                )?;
            }
            _ => {
                writeln!(f, "  Window time-delta: {}", self.window_time_delta.0)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averaging_type_bitfield_manipulation() {
        let mut avg = AveragingType::default();
        assert!(avg.is_none());
        assert_eq!(avg.as_u8(), 0);

        // Can combine multiple averaging types (Permission 9.6.1.1.2-1) + smoothing (Rule 9.6.1.1.2-9)
        avg.set_linear();
        avg.set_peak_hold();
        avg.set_smoothing();

        assert!(avg.linear());
        assert!(avg.peak_hold());
        assert!(avg.smoothing());
        assert!(!avg.min_hold());
        assert!(!avg.exponential());
        assert!(!avg.median());

        // Value = 1 (linear) | 2 (peak hold) | 32 (smoothing) = 35
        assert_eq!(avg.as_u8(), 35);
        assert_eq!(u8::from(avg), 35);

        // Unsetting works as expected
        avg.unset_peak_hold();
        assert!(!avg.peak_hold());
        assert_eq!(avg.as_u8(), 33);
        avg.set_peak_hold();

        // Setting on Spectrum struct round-trips
        let mut spec = Spectrum::default();
        spec.set_averaging_type(avg).unwrap();
        assert_eq!(spec.averaging_type().as_u8(), 35);
        assert!(spec.averaging_type().linear());
        assert!(spec.averaging_type().peak_hold());
        assert!(spec.averaging_type().smoothing());

        // Reserved bits (bits 6 and 7) are rejected
        let invalid = AveragingType(0b0100_0000);
        assert!(matches!(
            spec.set_averaging_type(invalid),
            Err(VitaError::ReservedField)
        ));
    }
}
