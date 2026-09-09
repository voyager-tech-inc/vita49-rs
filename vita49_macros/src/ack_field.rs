// SPDX-FileCopyrightText: 2025 The vita49-rs Authors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Ident, LitInt, Token};

struct AckFieldArgs {
    cif_num_literal: LitInt,
    _comma0: Token![,],
    ack_field: Ident,
}

impl Parse for AckFieldArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cif_num_literal = input.parse()?;
        let _comma0: Token![,] = input.parse()?;
        let ack_field = input.parse()?;
        Ok(AckFieldArgs {
            cif_num_literal,
            _comma0,
            ack_field,
        })
    }
}

pub fn ack_field(input: TokenStream) -> TokenStream {
    let AckFieldArgs {
        cif_num_literal,
        ack_field,
        ..
    } = parse2(input).expect("failed to parse macro input");

    let set_ack_field_fn = format_ident!("set_{}", ack_field);
    let unset_ack_field_fn = format_ident!("unset_{}", ack_field);

    let get_doc = format!(
        "Get the {ack_field} ACK level and ACK response. If `None` is returned, the field is unset."
    );
    let set_doc = format!(
        "Set the {ack_field} ACK level and ACK response. If `None` is passed, the field will be unset.\n\n\
        [`update_packet_size()`](Vrt::update_packet_size()) should be executed after running this method."
    );

    let cif_num = cif_num_literal.base10_parse::<u8>().unwrap();
    if cif_num == 0 {
        quote! {
            #[doc = #get_doc]
            fn #ack_field(&self) -> Option<(AckLevel, AckResponse)> {
                if let Some(fields) = self.eif0_fields() {
                    if let Some(response) = fields.#ack_field {
                        return Some((AckLevel::Error, response));
                    }
                }
                if let Some(fields) = self.wif0_fields() {
                    if let Some(response) = fields.#ack_field {
                        return Some((AckLevel::Warning, response));
                    }
                }
                None
            }

            #[doc = #set_doc]
            fn #set_ack_field_fn(&mut self, level: AckLevel, response: Option<AckResponse>) {
                match level {
                    AckLevel::Warning => {
                        if let Some(r) = response {
                            if self.wif0().is_none() {
                                *self.wif0_mut() = Some(Cif0::default());
                            }
                            self.wif0_mut().as_mut().unwrap().#set_ack_field_fn();

                            if self.wif0_fields().is_none() {
                                *self.wif0_fields_mut() = Some(Cif0AckFields::default());
                            }
                            self.wif0_fields_mut().as_mut().unwrap().#ack_field = Some(r);
                        } else {
                            let mut clear_wif = false;
                            let mut clear_wif_fields = false;
                            if let Some(w) = self.wif0_mut() {
                                w.#unset_ack_field_fn();
                                if w.empty() {
                                    clear_wif = true;
                                }
                            }
                            if let Some(f) = self.wif0_fields_mut() {
                                f.#ack_field = None;
                                if f.empty() {
                                    clear_wif_fields = true;
                                }
                            }
                            if clear_wif {
                                *self.wif0_mut() = None;
                            }
                            if clear_wif_fields {
                                *self.wif0_fields_mut() = None;
                            }
                        }
                    },
                    AckLevel::Error => {
                        if let Some(r) = response {
                            if self.eif0().is_none() {
                                *self.eif0_mut() = Some(Cif0::default());
                            }
                            self.eif0_mut().as_mut().unwrap().#set_ack_field_fn();
                            if self.eif0_fields().is_none() {
                                *self.eif0_fields_mut() = Some(Cif0AckFields::default());
                            }
                            self.eif0_fields_mut().as_mut().unwrap().#ack_field = Some(r);
                        } else {
                            let mut clear_wif = false;
                            let mut clear_wif_fields = false;
                            if let Some(w) = self.eif0_mut() {
                                w.#unset_ack_field_fn();
                                if w.empty() {
                                    clear_wif = true;
                                }
                            }
                            if let Some(f) = self.eif0_fields_mut() {
                                f.#ack_field = None;
                                if f.empty() {
                                    clear_wif_fields = true;
                                }
                            }
                            if clear_wif {
                                *self.eif0_mut() = None;
                            }
                            if clear_wif_fields {
                                *self.eif0_fields_mut() = None;
                            }
                        }
                    }
                };
            }
        }
    } else {
        let cif = format_ident!("Cif{}", cif_num);
        let cif_ack_fields = format_ident!("Cif{}AckFields", cif_num);
        let set_cif_enabled_fn = format_ident!("set_cif{}_enabled", cif_num);
        let unset_cif_enabled_fn = format_ident!("unset_cif{}_enabled", cif_num);
        let wif = format_ident!("wif{}", cif_num);
        let wif_mut = format_ident!("wif{}_mut", cif_num);
        let wif_fields = format_ident!("wif{}_fields", cif_num);
        let wif_fields_mut = format_ident!("wif{}_fields_mut", cif_num);

        let eif = format_ident!("eif{}", cif_num);
        let eif_mut = format_ident!("eif{}_mut", cif_num);
        let eif_fields = format_ident!("eif{}_fields", cif_num);
        let eif_fields_mut = format_ident!("eif{}_fields_mut", cif_num);

        quote! {
            #[doc = #get_doc]
            fn #ack_field(&self) -> Option<(AckLevel, AckResponse)> {
                if let Some(fields) = self.#eif_fields() {
                    if let Some(response) = fields.#ack_field {
                        return Some((AckLevel::Error, response));
                    }
                }
                if let Some(fields) = self.#wif_fields() {
                    if let Some(response) = fields.#ack_field {
                        return Some((AckLevel::Warning, response));
                    }
                }
                None
            }

            #[doc = #set_doc]
            fn #set_ack_field_fn(&mut self, level: AckLevel, response: Option<AckResponse>) {
                match level {
                    AckLevel::Warning => {
                        if let Some(r) = response {
                            if self.wif0().is_none() {
                                *self.wif0_mut() = Some(Cif0::default());
                            }
                            self.wif0_mut().as_mut().unwrap().#set_cif_enabled_fn();

                            if self.#wif().is_none() {
                                *self.#wif_mut() = Some(#cif::default());
                            }
                            self.#wif_mut().as_mut().unwrap().#set_ack_field_fn();

                            if self.#wif_fields().is_none() {
                                *self.#wif_fields_mut() = Some(#cif_ack_fields::default());
                            }
                            self.#wif_fields_mut().as_mut().unwrap().#ack_field = Some(r);
                        } else {
                            let mut clear_wif = false;
                            if let Some(f) = self.#wif_fields_mut().as_mut() {
                                f.#ack_field = None;
                            }
                            if let Some(w) = self.#wif_mut().as_mut() {
                                w.#unset_ack_field_fn();
                                if w.empty() {
                                    clear_wif = true;
                                }
                            }
                            if clear_wif {
                                *self.#wif_mut() = None;
                                *self.#wif_fields_mut() = None;
                                if let Some(w) = self.wif0_mut().as_mut() {
                                    w.#unset_cif_enabled_fn();
                                }
                            }
                        }
                    },
                    AckLevel::Error => {
                        if let Some(r) = response {
                            if self.eif0().is_none() {
                                *self.eif0_mut() = Some(Cif0::default());
                            }
                            self.eif0_mut().as_mut().unwrap().#set_cif_enabled_fn();

                            if self.#eif().is_none() {
                                *self.#eif_mut() = Some(#cif::default());
                            }
                            self.#eif_mut().as_mut().unwrap().#set_ack_field_fn();

                            if self.#eif_fields().is_none() {
                                *self.#eif_fields_mut() = Some(#cif_ack_fields::default());
                            }
                            self.#eif_fields_mut().as_mut().unwrap().#ack_field = Some(r);
                        } else {
                            let mut clear_eif = false;
                            if let Some(f) = self.#eif_fields_mut().as_mut() {
                                f.#ack_field = None;
                            }
                            if let Some(w) = self.#eif_mut().as_mut() {
                                w.#unset_ack_field_fn();
                                if w.empty() {
                                    clear_eif = true;
                                }
                            }
                            if clear_eif {
                                *self.#eif_mut() = None;
                                *self.#eif_fields_mut() = None;
                                if let Some(w) = self.eif0_mut().as_mut() {
                                    w.#unset_cif_enabled_fn();
                                }
                            }
                        }
                    }
                };
            }
        }
    }
}
