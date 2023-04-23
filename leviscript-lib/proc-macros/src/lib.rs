use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

/// Used on the Opcode enum. An opcode is basically an id paired with
/// arguments, and enums lend them selfes naturally to this usecase, however
/// opcodes will have to exist as a byte sequence, and for speed purposes
/// the binary representation will simply be achived via raw pointer cast.
/// The problem is, that the repr of an enum is as large as the largest variant,
/// and I don't want to waste that space. So the enum will be encoded as discriminant
/// plus the repr of it's data.
///
/// This Macro generates the following:
/// * a discriminant for each variant
/// * A function get_id() to get from variant to discriminant
/// * Self::to_bytes(&self) -> Vec<u8>
/// * Self::serialized_size(&self) -> usize
/// * Self::serialized_size_of(u16) -> usize
/// * unsafe Self::dispatch_discriminant(u16, *const u8, vm::Memory) -> *const u8
/// * get_body!(opcode) -> body_type macro

#[proc_macro_derive(OpCode)]
pub fn convert(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as ItemEnum);
    let variants: Vec<_> = input.variants.iter().collect();
    assert!(variants.len() <= 2 ^ 16, "Too many variants");
    let enum_name = input.ident;

    let variant_names = variants.iter().map(|v| &v.ident);
    let const_names: Vec<_> = variant_names.map(ident_to_upper).collect();
    let variant_infos: Vec<_> = variants
        .iter()
        .map(|v| {
            (
                &v.ident,
                match v.fields {
                    syn::Fields::Unit => None,
                    syn::Fields::Named(_) => {
                        panic!("Only variants with zero or one unnamed field allowed")
                    }
                    syn::Fields::Unnamed(ref fields) => {
                        assert!(
                            fields.unnamed.iter().len() == 1,
                            "Only variants with zero or one unnamed field allowed"
                        );
                        Some(&fields.unnamed.iter().next().unwrap().ty)
                    }
                },
            )
        })
        .collect();

    let consts = generate_u16_values_for_discriminants(&const_names);
    let get_id_fn_tokens = generate_get_id_fn(&const_names, &variant_infos);
    let serialize_tokens = generate_serialize_fn(&const_names, &variant_infos);
    let size_tokens = generate_size_fn(&const_names, &variant_infos);
    let dispatch_fn_tokens = generate_dispatch_discriminants_fn(&const_names);
    let get_body_macro_tokens = generate_get_body_macro(&variant_infos);

    quote! {
        #consts
        impl #enum_name {
            #get_id_fn_tokens
            #serialize_tokens
            #size_tokens
        }
        #dispatch_fn_tokens
        #get_body_macro_tokens
    }
    .into()
}

/// generates tokens that define a const u16 for each variant
fn generate_u16_values_for_discriminants(variants: &[syn::Ident]) -> TokenStream2 {
    variants
        .iter()
        .enumerate()
        .map(|(i, ident)| {
            let name = ident_to_upper(&ident);
            let i = i as u16;
            quote! {pub const #name: u16 = #i;}
        })
        .collect()
}

/// generates tokens for a function that converts a variant to a Vec<u8>
fn generate_serialize_fn(
    const_names: &[syn::Ident],
    variant_infos: &[(&syn::Ident, Option<&syn::Type>)],
) -> TokenStream2 {
    let arms = const_names.iter().zip(variant_infos.iter()).map(
        |(const_name, (var_name, args))| match args {
            None => quote! { crate::utils::any_as_u8_slice(&#const_name).to_vec() },
            Some(_) => quote! {
                Self::#var_name(data) => {
                    let mut base = crate::utils::any_as_u8_slice(&#const_name).to_vec();
                    base.extend_from_slice(crate::utils::any_as_u8_slice(data));
                    // everything should be at least word aligned, so if the size isn't even, the
                    // byte repr is padded
                    if base.len() % 2 == 1 {
                        base.push(0);
                    }
                    base
                }
            },
        },
    );

    let match_body: TokenStream2 = arms.collect();
    quote! {
        pub fn to_bytes(&self) -> Vec<u8> {
            match self {
                #match_body
            }
        }
    }
}

/// generates tokens for a function that returns the serialized size of a variant
/// which is 2 + the size of the argument type
fn generate_size_fn(
    const_names: &[syn::Ident],
    variant_infos: &[(&syn::Ident, Option<&syn::Type>)],
) -> TokenStream2 {
    let arms_u16: TokenStream2 = const_names
        .iter()
        .zip(variant_infos)
        .map(|(const_name, (_, types))| {
            let size = match types {
                None => quote! { 0 },
                Some(ty) => quote! { std::mem::size_of::<#ty>() },
            };
            quote! {
                #const_name => #size,
            }
        })
        .collect();

    quote! {
        pub fn serialized_size_of(disc: u16) -> usize {
            let size_unpadded = 2 + match disc {
                #arms_u16
                other => panic!("No enum id: {}", other),
             };
            if size_unpadded % 2 == 1 { size_unpadded + 1 } else { size_unpadded }
        }

        pub fn serialized_size(&self) -> usize {
            Self::serialized_size_of(self.get_id())
        }
    }
}

fn generate_get_id_fn(
    const_names: &[syn::Ident],
    variant_infos: &[(&syn::Ident, Option<&syn::Type>)],
) -> TokenStream2 {
    let arms: TokenStream2 = const_names
        .iter()
        .zip(variant_infos)
        .map(|(c_name, (var_ident, ty))| {
            if ty.is_some() {
                quote! { Self::#var_ident(_) => #c_name, }
            } else {
                quote! { Self::#var_ident => #c_name, }
            }
        })
        .collect();
    quote! {
        pub fn get_id(&self) -> u16 {
            match &self {
                #arms
            }
        }
    }
}

fn generate_dispatch_discriminants_fn(const_names: &[syn::Ident]) -> TokenStream2 {
    let arms: TokenStream2 = const_names
        .iter()
        .map(|const_name| {
            let target_fn_name = ident_map(|i| format!("exec_{}", i.to_lowercase()), const_name);
            quote! { #const_name => #target_fn_name(pc, mem), }
        })
        .collect();
    quote! {
        pub unsafe fn dispatch_discriminant(disc: u16, pc: *const u8, mem: &mut Memory) -> ExecResult {
            match disc {
                #arms
                _ => panic!("Unknown discriminant"),
            }
        }
    }
}

fn generate_get_body_macro(variant_infos: &[(&syn::Ident, Option<&syn::Type>)]) -> TokenStream2 {
    let arms: TokenStream2 = variant_infos
        .iter()
        .filter_map(|(var_name, ty)| {
            ty.map(|ty| {
                quote! {
                    (#var_name, $pc:expr) => { &*($pc as *const #ty) };
                }
            })
        })
        .collect();
    quote! {
        macro_rules! get_body {
            #arms
        }
        pub(crate) use get_body;
    }
}

fn ident_map(f: impl FnOnce(&str) -> String, i: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&f(&i.to_string()), i.span())
}

fn ident_to_upper(i: &syn::Ident) -> syn::Ident {
    ident_map(|s| s.to_uppercase(), i)
}