// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{
    cif7::Cif7Opts, prelude::*, Cif0AckFields, Cif0AckManipulators, Cif1AckFields,
    Cif1AckManipulators, Cif2AckFields, Cif2AckManipulators, Cif3AckFields, Cif3AckManipulators,
    ControlAckMode,
};
use deku::prelude::*;
use std::fmt;

/// ACK level indicating if the ACK is a warning or error.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AckLevel {
    /// This ACK represents a warning.
    Warning,
    /// This ACK represents an error.
    Error,
}

/// ACK data structure shared by validation and execution ACK packets.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, DekuRead, DekuWrite)]
#[deku(
    endian = "endian",
    ctx = "endian: deku::ctx::Endian, _cam: &ControlAckMode"
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ack {
    /// WIF0 indicator fields.
    #[deku(cond = "_cam.warning()")]
    wif0: Option<Cif0>,
    /// WIF1 indicator fields.
    #[deku(cond = "_cam.warning() && wif0.is_some() && wif0.unwrap().cif1_enabled()")]
    wif1: Option<Cif1>,
    /// WIF2 indicator fields.
    #[deku(cond = "_cam.warning() && wif0.is_some() && wif0.unwrap().cif2_enabled()")]
    wif2: Option<Cif2>,
    /// WIF3 indicator fields.
    #[deku(cond = "_cam.warning() && wif0.is_some() && wif0.unwrap().cif3_enabled()")]
    wif3: Option<Cif3>,
    /// WIF7 indicator fields.
    #[deku(cond = "_cam.warning() && wif0.is_some() && wif0.unwrap().field_attributes_enabled()")]
    pub wif7: Option<Cif7>,

    /// EIF0 indicator fields.
    #[deku(cond = "_cam.error()")]
    eif0: Option<Cif0>,
    /// EIF1 indicator fields.
    #[deku(cond = "_cam.error() && eif0.is_some() && eif0.unwrap().cif1_enabled()")]
    eif1: Option<Cif1>,
    /// EIF2 indicator fields.
    #[deku(cond = "_cam.error() && eif0.is_some() && eif0.unwrap().cif2_enabled()")]
    eif2: Option<Cif2>,
    /// EIF3 indicator fields.
    #[deku(cond = "_cam.error() && eif0.is_some() && eif0.unwrap().cif3_enabled()")]
    eif3: Option<Cif3>,
    /// EIF7 indicator fields.
    #[deku(cond = "_cam.error() && eif0.is_some() && eif0.unwrap().field_attributes_enabled()")]
    pub eif7: Option<Cif7>,

    #[deku(
        cond = "wif0.is_some()",
        ctx = "wif0.as_ref(), Cif7Opts::from(wif7.as_ref())"
    )]
    wif0_fields: Option<Cif0AckFields>,
    #[deku(
        cond = "wif1.is_some()",
        ctx = "wif1.as_ref(), Cif7Opts::from(wif7.as_ref())"
    )]
    wif1_fields: Option<Cif1AckFields>,
    #[deku(
        cond = "wif2.is_some()",
        ctx = "wif2.as_ref(), Cif7Opts::from(wif7.as_ref())"
    )]
    wif2_fields: Option<Cif2AckFields>,
    #[deku(
        cond = "wif3.is_some()",
        ctx = "wif3.as_ref(), Cif7Opts::from(wif7.as_ref())"
    )]
    wif3_fields: Option<Cif3AckFields>,

    #[deku(
        cond = "eif0.is_some()",
        ctx = "eif0.as_ref(), Cif7Opts::from(eif7.as_ref())"
    )]
    eif0_fields: Option<Cif0AckFields>,
    #[deku(
        cond = "eif1.is_some()",
        ctx = "eif1.as_ref(), Cif7Opts::from(eif7.as_ref())"
    )]
    eif1_fields: Option<Cif1AckFields>,
    #[deku(
        cond = "eif2.is_some()",
        ctx = "eif2.as_ref(), Cif7Opts::from(eif7.as_ref())"
    )]
    eif2_fields: Option<Cif2AckFields>,
    #[deku(
        cond = "eif3.is_some()",
        ctx = "eif3.as_ref(), Cif7Opts::from(eif7.as_ref())"
    )]
    eif3_fields: Option<Cif3AckFields>,
}

impl Ack {
    /// Get the ACK size (in 32-bit words).
    pub fn size_words(&self) -> u16 {
        let mut ret = 0;
        for ind in [
            self.wif0.is_some(),
            self.wif1.is_some(),
            self.wif2.is_some(),
            self.wif3.is_some(),
            self.wif7.is_some(),
            self.eif0.is_some(),
            self.eif1.is_some(),
            self.eif2.is_some(),
            self.eif3.is_some(),
            self.eif7.is_some(),
        ] {
            if ind {
                ret += 1;
            }
        }
        if let Some(f) = &self.wif0_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.wif1_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.wif2_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.wif3_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.eif0_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.eif1_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.eif2_fields {
            ret += f.size_words();
        }
        if let Some(f) = &self.eif3_fields {
            ret += f.size_words();
        }
        ret
    }
}

impl Cif0AckManipulators for Ack {
    fn wif0(&self) -> Option<&Cif0> {
        self.wif0.as_ref()
    }
    fn wif0_mut(&mut self) -> &mut Option<Cif0> {
        &mut self.wif0
    }
    fn wif0_fields(&self) -> Option<&Cif0AckFields> {
        self.wif0_fields.as_ref()
    }
    fn wif0_fields_mut(&mut self) -> &mut Option<Cif0AckFields> {
        &mut self.wif0_fields
    }

    fn eif0(&self) -> Option<&Cif0> {
        self.eif0.as_ref()
    }
    fn eif0_mut(&mut self) -> &mut Option<Cif0> {
        &mut self.eif0
    }
    fn eif0_fields(&self) -> Option<&Cif0AckFields> {
        self.eif0_fields.as_ref()
    }
    fn eif0_fields_mut(&mut self) -> &mut Option<Cif0AckFields> {
        &mut self.eif0_fields
    }
}

macro_rules! impl_cif_ack {
    (
        $trait:ident,
        $wif:ident, $wif_mut:ident, $wif_fields:ident, $wif_fields_mut:ident, $cif:ident, $fields:ident,
        $eif:ident, $eif_mut:ident, $eif_fields:ident, $eif_fields_mut:ident
    ) => {
        impl $trait for Ack {
            fn wif0(&self) -> Option<&Cif0> {
                self.wif0.as_ref()
            }
            fn wif0_mut(&mut self) -> &mut Option<Cif0> {
                &mut self.wif0
            }
            fn wif0_fields(&self) -> Option<&Cif0AckFields> {
                self.wif0_fields.as_ref()
            }
            fn wif0_fields_mut(&mut self) -> &mut Option<Cif0AckFields> {
                &mut self.wif0_fields
            }

            fn eif0(&self) -> Option<&Cif0> {
                self.eif0.as_ref()
            }
            fn eif0_mut(&mut self) -> &mut Option<Cif0> {
                &mut self.eif0
            }
            fn eif0_fields(&self) -> Option<&Cif0AckFields> {
                self.eif0_fields.as_ref()
            }
            fn eif0_fields_mut(&mut self) -> &mut Option<Cif0AckFields> {
                &mut self.eif0_fields
            }

            fn $wif(&self) -> Option<&$cif> {
                self.$wif.as_ref()
            }
            fn $wif_mut(&mut self) -> &mut Option<$cif> {
                &mut self.$wif
            }
            fn $wif_fields(&self) -> Option<&$fields> {
                self.$wif_fields.as_ref()
            }
            fn $wif_fields_mut(&mut self) -> &mut Option<$fields> {
                &mut self.$wif_fields
            }

            fn $eif(&self) -> Option<&$cif> {
                self.$eif.as_ref()
            }
            fn $eif_mut(&mut self) -> &mut Option<$cif> {
                &mut self.$eif
            }
            fn $eif_fields(&self) -> Option<&$fields> {
                self.$eif_fields.as_ref()
            }
            fn $eif_fields_mut(&mut self) -> &mut Option<$fields> {
                &mut self.$eif_fields
            }
        }
    };
}

impl_cif_ack!(
    Cif1AckManipulators,
    wif1,
    wif1_mut,
    wif1_fields,
    wif1_fields_mut,
    Cif1,
    Cif1AckFields,
    eif1,
    eif1_mut,
    eif1_fields,
    eif1_fields_mut
);
impl_cif_ack!(
    Cif2AckManipulators,
    wif2,
    wif2_mut,
    wif2_fields,
    wif2_fields_mut,
    Cif2,
    Cif2AckFields,
    eif2,
    eif2_mut,
    eif2_fields,
    eif2_fields_mut
);
impl_cif_ack!(
    Cif3AckManipulators,
    wif3,
    wif3_mut,
    wif3_fields,
    wif3_fields_mut,
    Cif3,
    Cif3AckFields,
    eif3,
    eif3_mut,
    eif3_fields,
    eif3_fields_mut
);

impl fmt::Display for Ack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ACK")?;
        // TODO: improve printout
        writeln!(f, "{self:#?}")?;
        Ok(())
    }
}
