// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Ident, LitInt, Token};

struct TodoCifFieldArgs {
    cif_field: Ident,
    _comma: Token![,],
    bit: LitInt,
    _comma2: Token![,],
    cif_num: LitInt,
}

impl Parse for TodoCifFieldArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cif_field = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let bit = input.parse()?;
        let _comma2: Token![,] = input.parse()?;
        let cif_num = input.parse()?;
        Ok(TodoCifFieldArgs {
            cif_field,
            _comma,
            bit,
            _comma2,
            cif_num,
        })
    }
}

pub fn todo_cif_field(input: TokenStream) -> TokenStream {
    let TodoCifFieldArgs {
        cif_field,
        bit,
        cif_num,
        ..
    } = parse2(input).expect("failed to parse macro input");

    let set = format_ident!("set_{}", cif_field);
    let unset = format_ident!("unset_{}", cif_field);
    let mask = format_ident!("UNSUPPORTED_{}", cif_field.to_string().to_uppercase());

    let mask_doc = format!("Bit mask for the unimplemented {cif_field} CIF field");
    let get_doc = format!("Panics if the {cif_field} CIF field bit is set, false otherwise");
    let set_doc = format!("Sets the {cif_field} CIF field bit");
    let unset_doc = format!("Unsets the {cif_field} CIF field bit");

    quote! {
        #[doc = #mask_doc]
        const #mask: u32 = 1 << #bit;

        #[doc = #get_doc]
        pub fn #cif_field(&self) -> bool {
            if self.0 & (1 << #bit) != 0 {
                panic!(
                    "CIF{} bit {} ({}) is set, but support for this field is unimplemented",
                    #cif_num,
                    #bit,
                    stringify!(#cif_field),
                );
            } else {
                false
            }
        }
        #[doc = #set_doc]
        pub fn #set(&mut self) {
            self.0 |= 1 << #bit;
        }
        #[doc = #unset_doc]
        pub fn #unset(&mut self) {
            self.0 &= !(1 << #bit);
        }
    }
}
