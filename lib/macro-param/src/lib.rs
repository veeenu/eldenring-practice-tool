use heck::AsSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::*;

#[proc_macro_derive(ParamStruct, attributes(bitflag))]
pub fn macro_param(t: TokenStream) -> TokenStream {
    let input = parse_macro_input!(t as DeriveInput);
    let name = input.ident;
    let fields_punct = match input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("Only structs with named fields can be annotated"),
    };

    let fields_with_bitfields = fields_punct
        .iter()
        .map(|field| {
            (
                field,
                field
                    .attrs
                    .iter()
                    .map(|attr| {
                        let meta_list = match attr.parse_meta() {
                            Ok(Meta::List(meta_list)) if meta_list.path.is_ident("bitflag") => {
                                meta_list
                            },
                            other => unimplemented!("Unimplemented attribute {:#?}", other),
                        };

                        match (&meta_list.nested[0], &meta_list.nested[1]) {
                            (
                                NestedMeta::Meta(Meta::Path(path)),
                                NestedMeta::Lit(Lit::Int(fieldno)),
                            ) => {
                                let bitfield_name = path.get_ident().unwrap().to_owned();
                                let set_ident = format_ident!(
                                    "set_{}",
                                    AsSnakeCase(bitfield_name.to_string()).to_string()
                                );
                                let get_ident = format_ident!(
                                    "{}",
                                    AsSnakeCase(bitfield_name.to_string()).to_string()
                                );
                                (
                                    bitfield_name,
                                    fieldno.base10_parse::<u8>().unwrap(),
                                    set_ident,
                                    get_ident,
                                )
                            },
                            other => panic!("Wrong attribute parameters: {:#?}", other),
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let bitfield_methods = fields_with_bitfields.iter().flat_map(|(field, bitfield_spec)| {
        bitfield_spec.iter().map(|(_, field_idx, set_ident, get_ident)| {
            let ident = format_ident!("{}", field.ident.as_ref().unwrap());
            quote! {
                #[allow(unused)]
                pub fn #get_ident(&self) -> bool {
                    self.#ident & (1 << #field_idx) != 0
                }

                #[allow(unused)]
                pub fn #set_ident(&mut self, state: bool) {
                    if state {
                        self.#ident |= 1 << #field_idx;
                    } else {
                        self.#ident &= !(1 << #field_idx);
                    }
                }
            }
        })
    });

    let field_visit = fields_with_bitfields
        .iter()
        .filter_map(|(field, bitfield_spec)| match field {
            &Field { ident: Some(ident), ty: Type::Path(TypePath { path, .. }), .. } => {
                let ty_ident = path.segments[0].ident.to_string();
                match ty_ident.as_str() {
                    "u8" if !bitfield_spec.is_empty() => {
                        let bitfield_visit = bitfield_spec.iter().map(
                            |(bitfield_name, _, set_bitfield, get_bitfield)| {
                                quote! {
                                    let mut b = self.#get_bitfield();
                                    t.visit_bool(stringify!(#bitfield_name), &mut b);
                                    self.#set_bitfield(b);
                                }
                            },
                        );

                        Some(quote! {
                            #(#bitfield_visit)*
                        })
                    },
                    "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "f32" => {
                        let ident = format_ident!("{}", ident);
                        let visit_ty = format_ident!("visit_{}", ty_ident);
                        Some(quote! {
                            t.#visit_ty(stringify!(#ident), &mut self.#ident);
                        })
                    },
                    other => panic!("Unrecognized type {:#?}", other),
                }
            },
            &Field {
                ident: Some(_),
                ty:
                    Type::Array(TypeArray {
                        elem: _,
                        len: Expr::Lit(ExprLit { lit: Lit::Int(_), .. }),
                        ..
                    }),
                ..
            } => {
                // Just ignore array fields
                None
            },
            field => {
                panic!("Unrecognized field {:#?}", field);
            },
        })
        .collect::<Vec<_>>();

    let visit = quote! {
        fn visit<T: ParamVisitor + ?Sized>(&mut self, t: &mut T) {
            #(#field_visit)*
        }
    };

    let get_name_snake_case = format_ident!("get_{}", AsSnakeCase(name.to_string()).to_string());
    quote! {
        impl #name {
            #(#bitfield_methods)*
        }

        impl ParamStruct for #name {
            #visit
        }

        impl Params {
            pub unsafe fn #get_name_snake_case(&self) -> Option<impl Iterator<Item = Param<#name>>> {
                self.iter_param::<#name>(stringify!(#name))
            }
        }
    }
    .into()
}
