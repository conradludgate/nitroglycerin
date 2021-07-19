use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Visibility;

use crate::{
    attr::{FieldAttr, TableAttr},
    convert::ConvertBuilder,
    get::GetBuilder,
    query::{Column, QueryBuilder},
};

pub struct Table {
    vis: Visibility,
    name: syn::Ident,
    // fields: Vec<NamedField>,
    // args: Vec<syn::Ident>,
    attrs: TableAttr,
    columns: Vec<Column>,
    partition_key: Column,
    sort_key: Option<Column>,
    // generic: syn::Type,
}

impl Table {
    pub fn new(vis: Visibility, name: syn::Ident, generics: syn::Generics, attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<Self> {
        // let args = generics.type_params().cloned().map(|tp| tp.ident).collect();
        let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;
        let attrs = TableAttr::parse_attrs(attrs)?;
        // let generic = parse_or(&attrs.parse_type);

        let columns: Vec<_> = fields
            .iter()
            .map(|f| Column {
                name: f.attrs.rename.as_ref().map(|l| l.value()).unwrap_or_else(|| f.name.to_string()),
                ident: f.name.clone(),
                ty: f.ty.clone(),
            })
            .collect();

        let partition_key = fields
            .iter()
            .find_map(|f| {
                f.attrs.partition_key?;
                Some(Column {
                    name: f.attrs.rename.as_ref().map(|l| l.value()).unwrap_or_else(|| f.name.to_string()),
                    ident: f.name.clone(),
                    ty: f.ty.clone(),
                })
            })
            .ok_or(syn::Error::new(Span::call_site(), "table needs a partition key"))?;

        let sort_key = fields.iter().find_map(|f| {
            f.attrs.sort_key?;
            Some(Column {
                name: f.attrs.rename.as_ref().map(|l| l.value()).unwrap_or_else(|| f.name.to_string()),
                ident: f.name.clone(),
                ty: f.ty.clone(),
            })
        });

        Ok(Table {
            vis,
            attrs,
            name,
            columns,
            partition_key,
            sort_key,
            // args,
            // fields,
            // generic,
        })
    }
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Table {
            vis,
            name,
            // fields,
            // args,
            attrs,
            columns,
            partition_key,
            sort_key,
            // generic,
        } = self;

        ConvertBuilder {
            ident: name.to_owned(),
            columns: columns.clone(),
        }
        .to_tokens(tokens);

        GetBuilder {
            vis: vis.to_owned(),
            table_name: attrs.table_name.clone(),
            output: name.to_owned(),
            partition_key: partition_key.clone(),
            sort_key: sort_key.clone(),
        }
        .to_tokens(tokens);

        QueryBuilder {
            vis: vis.to_owned(),
            table_name: attrs.table_name.clone(),
            index_name: None,
            output: name.to_owned(),
            partition_key: partition_key.clone(),
            sort_key: sort_key.clone(),
        }
        .to_tokens(tokens);
    }
}

#[derive(Debug)]
struct NamedField {
    attrs: FieldAttr,
    name: syn::Ident,
    ty: syn::Type,
}

impl TryFrom<syn::Field> for NamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field { ident, attrs, ty, .. } = field;
        let attrs = FieldAttr::parse_attrs(attrs)?;
        Ok(NamedField { attrs, name: ident.unwrap(), ty })
    }
}
