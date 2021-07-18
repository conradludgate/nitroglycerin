use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

#[macro_use]
extern crate derive_builder;

mod attr;
mod get;
mod query;
mod split_by;
mod table;

#[proc_macro_derive(Table, attributes(nitro))]
pub fn derive_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let DeriveInput { attrs, vis, ident, generics, data } = input;

    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => match table::Table::new(vis, ident, generics, attrs, fields) {
                Ok(t) => t.into_token_stream(),
                Err(e) => e.to_compile_error(),
            },
            syn::Fields::Unnamed(_) => syn::Error::new(span, "tuple structs not supported").into_compile_error(),
            syn::Fields::Unit => syn::Error::new(span, "unit structs not supported").into_compile_error(),
        },
        syn::Data::Enum(_) => syn::Error::new(span, "enums not supported").into_compile_error(),
        syn::Data::Union(_) => syn::Error::new(span, "unions not supported").into_compile_error(),
    }
    .into()
}
