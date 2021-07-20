use std::convert::TryFrom;

use attr::FieldAttr;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, DeriveInput};

#[macro_use]
extern crate derive_builder;

mod attr;
mod convert;
mod key;
mod query;
mod split_by;

type Parser = fn(vis: syn::Visibility, name: syn::Ident, generics: syn::Generics, attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<proc_macro2::TokenStream>;
fn derive(input: TokenStream, parser: Parser) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let DeriveInput { attrs, vis, ident, generics, data } = input;

    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => match parser(vis, ident, generics, attrs, fields) {
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

/// Implement a strongly typed key builder. This is used to setup get requests
#[proc_macro_derive(Key, attributes(nitro))]
pub fn derive_key(input: TokenStream) -> TokenStream {
    derive(input, key::derive)
}

/// Implement a strongly typed query builder. This is used to setup query requests
#[proc_macro_derive(Query, attributes(nitro))]
pub fn derive_query(input: TokenStream) -> TokenStream {
    derive(input, query::derive)
}

#[proc_macro_derive(Attributes, attributes(nitro))]
pub fn derive_convert(input: TokenStream) -> TokenStream {
    derive(input, convert::derive)
}

#[derive(Clone, Copy)]
struct D;
impl From<D> for syn::Ident {
    fn from(_: D) -> Self {
        parse_quote!(__NitroglycerinDynamoDBClient)
    }
}
impl quote::ToTokens for D {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        syn::Ident::from(*self).to_tokens(tokens);
    }
}

#[derive(Clone)]
struct NamedField {
    pub attrs: FieldAttr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl TryFrom<syn::Field> for NamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field { ident, attrs, ty, .. } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(Self { attrs, name: ident.unwrap(), ty })
    }
}

#[derive(Clone)]
struct Column {
    pub ident: syn::Ident,
    pub name: String,
    pub ty: syn::Type,
}

impl From<NamedField> for Column {
    fn from(f: NamedField) -> Self {
        Self {
            name: f.attrs.rename.as_ref().map(|l| l.value()).unwrap_or_else(|| f.name.to_string()),
            ident: f.name,
            ty: f.ty,
        }
    }
}
