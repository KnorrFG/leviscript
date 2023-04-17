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
/// * Self::to_bytes(&self) -> Vec<u8>
/// * Self::serialized_size(&self) -> usize
/// * unsafe Self::from_ptr(p: const * u8) -> Self
#[proc_macro_derive(ByteConvertible)]
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
    let serialize_tokens = generate_serialize_fn(&const_names, &variant_infos);
    let size_tokens = generate_size_fn(&variant_infos);
    let from_ptr_tokens = generate_from_ptr_fn(&const_names, &variant_infos);

    quote! {
        #consts
        impl #enum_name {
            #serialize_tokens
            #size_tokens
            #from_ptr_tokens
        }
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
fn generate_size_fn(variant_infos: &[(&syn::Ident, Option<&syn::Type>)]) -> TokenStream2 {
    let arms = variant_infos
        .iter()
        .map(|(var_name, var_type)| match var_type {
            None => quote! { #var_name => 0, },
            Some(ty) => quote! { Self::#var_name(_) => std::mem::size_of::<#ty>(), },
        });
    let match_body: TokenStream2 = arms.collect();
    quote! {
        pub fn serialized_size(&self) -> usize {
            let size_unpadded = 2 + match self {
                #match_body
            };
            if size_unpadded % 2 == 1 { size_unpadded + 1 } else { size_unpadded }
        }
    }
}

/// generates an unsafe function that converts from a const * u8 to Self
fn generate_from_ptr_fn(
    const_names: &[syn::Ident],
    variant_infos: &[(&syn::Ident, Option<&syn::Type>)],
) -> TokenStream2 {
    let arms: TokenStream2 = const_names
        .iter()
        .zip(variant_infos.iter())
        .map(|(const_name, (var_name, var_type))| {
            let rhs = if let Some(ty) = var_type {
                quote! { Self::#var_name(*(p.offset(2) as *const #ty)), }
            } else {
                quote! { Self::#var_name, }
            };
            quote! { #const_name => #rhs }
        })
        .collect();
    quote! {
        pub unsafe fn from_ptr(p: *const u8) -> Self {
            match *(p as *const u16) {
                #arms
                unknown => panic!("Unknown discriminant found: {}, this is a bug.", unknown),
            }
        }
    }
}

fn ident_to_upper(i: &syn::Ident) -> syn::Ident {
    syn::Ident::new(&i.to_string().to_uppercase(), i.span())
}
