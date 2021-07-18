
use std::convert::{TryFrom};

use crate::{attr::{TableAttr, FieldAttr}, get::GetBuilder, query::{Column, QueryBuilder}};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{VisPublic, Visibility, parse, parse2};

pub struct Table {
    vis: Visibility,
    name: syn::Ident,
    fields: Vec<NamedField>,
    args: Vec<syn::Ident>,
    attrs: TableAttr,
    // generic: syn::Type,
}

impl Table {
    pub fn new(
        vis: Visibility,
        name: syn::Ident,
        generics: syn::Generics,
        attrs: Vec<syn::Attribute>,
        fields: syn::FieldsNamed,
    ) -> syn::Result<Self> {
        let args = generics.type_params().cloned().map(|tp| tp.ident).collect();
        let fields = fields
            .named
            .into_iter()
            .map(NamedField::try_from)
            .collect::<syn::Result<_>>()?;
        let attrs = TableAttr::parse_attrs(attrs)?;
        // let generic = parse_or(&attrs.parse_type);

        Ok(Table {
            vis,
            attrs,
            name,
            args,
            fields,
            // generic,
        })
    }
}


impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Table {
            vis,
            name,
            fields,
            args,
            attrs,
            // generic,
        } = self;

        GetBuilder {
            vis: vis.to_owned(),
            table_name: attrs.table_name.value(),
            output: name.to_owned(),
            partition_key: Column {
                ident: format_ident!("id"),
                name: "id".to_owned(),
                ty: parse2(quote!{ String }).unwrap()
            },
            sort_key: Some(Column {
                ident: format_ident!("sort"),
                name: "sort".to_owned(),
                ty: parse2(quote!{ u32 }).unwrap()
            }),
        }.to_tokens(tokens);

        QueryBuilder {
            vis: vis.to_owned(),
            table_name: attrs.table_name.value(),
            index_name: Some("FooIndex".to_owned()),
            output: name.to_owned(),
            partition_key: Column {
                ident: format_ident!("id"),
                name: "id".to_owned(),
                ty: parse2(quote!{ String }).unwrap()
            },
            sort_key: Some(Column {
                ident: format_ident!("sort"),
                name: "sort".to_owned(),
                ty: parse2(quote!{ u32 }).unwrap()
            }),
        }.to_tokens(tokens);
    }
}

struct NamedField {
    attrs: FieldAttr,
    name: syn::Ident,
    ty: syn::Type,
}

impl TryFrom<syn::Field> for NamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field {
            ident, attrs, ty, ..
        } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(NamedField {
            attrs,
            name: ident.unwrap(),
            ty,
        })
    }
}
