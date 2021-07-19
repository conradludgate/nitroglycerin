use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Visibility;

use crate::{
    attr::{FieldAttr, TableAttr},
    convert::ConvertBuilder,
    get::GetBuilder,
    query::QueryBuilder,
};

pub struct Table {
    convert: ConvertBuilder,
    get: GetBuilder,
    query: QueryBuilder,
}

impl Table {
    pub fn new(vis: Visibility, name: syn::Ident, generics: syn::Generics, attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<Self> {
        let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;
        let attrs = TableAttr::parse_attrs(attrs)?;

        let columns: Vec<_> = fields.clone().into_iter().map(Column::from).collect();

        let partition_key: Column = fields
            .iter()
            .find_map(|f| f.attrs.partition_key.map(|_| f.clone().into()))
            .ok_or_else(|| syn::Error::new(Span::call_site(), "table needs a partition key"))?;

        let sort_key = fields.iter().find_map(|f| f.attrs.sort_key.map(|_| f.clone().into()));

        let convert = ConvertBuilder::new(name.to_owned(), generics.clone(), columns.clone());

        let get = GetBuilder::new(vis.to_owned(), attrs.table_name.clone(), name.to_owned(), generics.clone(), partition_key.clone(), sort_key.clone());
        let query = QueryBuilder::new(
            vis.to_owned(),
            attrs.table_name.clone(),
            None,
            name.to_owned(),
            generics.clone(),
            partition_key.clone(),
            sort_key.clone(),
        );

        Ok(Table { convert, get, query })
    }
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { convert, get, query } = self;
        convert.to_tokens(tokens);
        get.to_tokens(tokens);
        query.to_tokens(tokens);
    }
}

#[derive(Clone, Debug)]
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
        Ok(Self { attrs, name: ident.unwrap(), ty })
    }
}

#[derive(Clone)]
pub struct Column {
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
