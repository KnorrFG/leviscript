use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ItemEnum, ItemStruct};

use std::iter;

pub fn ast_node_impl(tokens: TokenStream) -> TokenStream {
    let def = parse_macro_input!(tokens as syn::Item);
    match def {
        syn::Item::Enum(en_def) => enum_impl(en_def),
        syn::Item::Struct(s_def) => struct_impl(s_def),
        _ => panic!("This macro can only be used with structs and enums"),
    }
}

pub fn enum_impl(def: ItemEnum) -> TokenStream {
    let enum_name = &def.ident;

    let arms: Vec<_> = def
        .variants
        .iter()
        .map(|v| {
            let syn::Fields::Unnamed(fields) = &v.fields else {
                panic!("This macro on supports enums variants with a signle unnamed field");
            };
            assert!(
                fields.unnamed.iter().count() == 1,
                "This macro on supports enums variants with a signle unnamed field"
            );
            let var_name = &v.ident;
            (
                quote! { Self::#var_name(x) => x.get_id(), },
                quote! { Self::#var_name(x) => x.children(), },
                quote! { Self::#var_name(x) => x.get_node_ref(), },
            )
        })
        .collect();

    let (id_arms, ch_arms, ref_arms): (TokenStream2, TokenStream2, TokenStream2) =
        itertools::multiunzip(arms);

    quote! {
        impl AstNode for #enum_name {
            fn get_id(&self) -> EnvironmentIdentifier {
                match self {
                    #id_arms
                }
            }

            fn children<'a>(&'a self) -> Box<dyn Iterator<Item = AstNodeRef<'a>> + '_> {
                match self {
                    #ch_arms
                }
            }

            fn get_node_ref(&self) -> AstNodeRef {
                match self {
                    #ref_arms
                }
            }
        }

    }
    .into()
}

fn struct_impl(st_def: ItemStruct) -> TokenStream {
    let name = &st_def.ident;
    match st_def.fields {
        syn::Fields::Unnamed(fields) => unnamed_st_impl(name, fields),
        syn::Fields::Named(fields) => named_st_impl(name, fields),
        syn::Fields::Unit => panic!("Unit structs aren't allowed"),
    }
}

fn unnamed_st_impl(name: &syn::Ident, fields: syn::FieldsUnnamed) -> TokenStream {
    let fields: Vec<_> = fields.unnamed.iter().collect();
    assert!(
        fields.len() > 0,
        "a struct with unnamed fields must always have a usize as first field."
    );
    let usize_t: syn::Type = parse_quote!(usize);
    assert!(fields[0].ty == usize_t, "The first field is no usize");
    let get_id_impl = quote! {
        fn get_id(&self) -> EnvironmentIdentifier {
            EnvironmentIdentifier::AstId(self.0)
        }
    };

    let child_attr: syn::Attribute = parse_quote!(#[child]);
    let children_attr: syn::Attribute = parse_quote!(#[children]);

    // this creates two iterators. one with the indices of all fields that are anotated with
    // #[child], and one for the fields annotated with #[children]
    let init: (
        Box<dyn Iterator<Item = usize>>,
        Box<dyn Iterator<Item = usize>>,
    ) = (Box::new(iter::empty()), Box::new(iter::empty()));
    let (child_ids, children_ids) = fields.into_iter().enumerate().skip(1).fold(
        init,
        |(child_iter, children_iter), (i, field)| {
            if field
                .attrs
                .iter()
                .find(|attr| **attr == child_attr)
                .is_some()
            {
                (Box::new(child_iter.chain(iter::once(i))), children_iter)
            } else if field.attrs.iter().find(|a| **a == children_attr).is_some() {
                (child_iter, Box::new(children_iter.chain(iter::once(i))))
            } else {
                (child_iter, children_iter)
            }
        },
    );

    // These iterators are then used to build the iterator that is returned by AstNode::children()
    let children_iter = child_ids.map(syn::Index::from).fold(
        quote!(std::iter::empty()),
        |it, child_id| quote!(#it.chain(std::iter::once(self.#child_id.as_ref().into()))),
    );
    let children_iter = children_ids.map(syn::Index::from).fold(
        children_iter,
        |it, children_id| quote!(#it.chain(self.#children_id.iter().map(AstNodeRef::from))),
    );
    let children_impl = quote! {
        fn children<'a>(&'a self) -> Box<dyn Iterator<Item = AstNodeRef<'a>> + '_> {
            Box::new(#children_iter)
        }
    };

    quote! {
        impl AstNode for #name {
            #get_id_impl
            #children_impl

            fn get_node_ref(&self) -> AstNodeRef {
                self.into()
            }
        }
    }
    .into()
}

fn named_st_impl(name: &syn::Ident, fields: syn::FieldsNamed) -> TokenStream {
    let fields: Vec<_> = fields.named.iter().collect();
    let usize_t: syn::Type = parse_quote!(usize);
    let id_field = fields
        .iter()
        .find(|f| f.ident.as_ref().unwrap().to_string() == "id" && f.ty == usize_t);
    assert!(
        id_field.is_some(),
        "a struct with named fields must always have a id: usize field."
    );
    let get_id_impl = quote! {
        fn get_id(&self) -> EnvironmentIdentifier {
            EnvironmentIdentifier::AstId(self.id)
        }
    };

    let child_attr: syn::Attribute = parse_quote!(#[child]);
    let children_attr: syn::Attribute = parse_quote!(#[children]);

    let init: (
        Box<dyn Iterator<Item = &syn::Ident>>,
        Box<dyn Iterator<Item = &syn::Ident>>,
    ) = (Box::new(iter::empty()), Box::new(iter::empty()));
    let (child_ids, children_ids) =
        fields
            .into_iter()
            .fold(init, |(child_iter, children_iter), field| {
                if field
                    .attrs
                    .iter()
                    .find(|attr| **attr == child_attr)
                    .is_some()
                {
                    (
                        Box::new(child_iter.chain(iter::once(field.ident.as_ref().unwrap()))),
                        children_iter,
                    )
                } else if field.attrs.iter().find(|a| **a == children_attr).is_some() {
                    (
                        child_iter,
                        Box::new(children_iter.chain(iter::once(field.ident.as_ref().unwrap()))),
                    )
                } else {
                    (child_iter, children_iter)
                }
            });

    let children_iter = child_ids.fold(
        quote!(std::iter::empty()),
        |it, child_field| quote!(#it.chain(std::iter::once(self.#child_field.as_ref().into()))),
    );
    let children_iter = children_ids.fold(
        children_iter,
        |it, children_field| quote!(#it.chain(self.#children_field.iter().map(AstNodeRef::from))),
    );
    let children_impl = quote! {
        fn children<'a>(&'a self) -> Box<dyn Iterator<Item = AstNodeRef<'a>> + '_> {
            Box::new(#children_iter)
        }
    };
    quote! {
        impl AstNode for #name {
            #get_id_impl
            #children_impl

            fn get_node_ref(&self) -> AstNodeRef {
                self.into()
            }
        }
    }
    .into()
}
