//! High level dynamodb crate
//!
//! ```ignore
//! use nitroglycerin::{Attributes, Key, Query, Table, DynamoDb, dynamodb::DynamoDbClient};
//! use rusoto_core::Region;
//!
//! #[derive(Debug, PartialEq, Attributes, Key, Query)]
//! struct Employee {
//!     #[nitro(partition_key)]
//!     id: String,
//!     #[nitro(rename = "firstName")]
//!     name: String,
//!     joined: i64,
//!     left: Option<i64>,
//! }
//!
//! impl Table for Employee {
//!     fn table_name() -> String {
//!         "Employees".to_string()
//!     }
//! }
//!
//! #[derive(Debug, PartialEq, Attributes, Query)]
//! struct EmployeeNameIndex {
//!     #[nitro(partition_key, rename = "firstName")]
//!     name: String,
//!     #[nitro(sort_key)]
//!     joined: i64,
//! }
//!
//! impl IndexTable for EmployeeNameIndex {
//!     type Table = Employees;
//!     fn index_name() -> Option<String> {
//!         Some("EmployeeNamesIndex".to_string())
//!     }
//! }
//!
//! let client = DynamoDbClient::new(Region::default());
//!
//! let employee: Option<Employee> = client.get::<Employee>()
//!    .id("emp_1") // get the employee with id "emp_1"
//!    .execute().await?;
//!
//! let new_employee = Employee {
//!    id: "emp_1234".into(),
//!    name: "Conrad".into(),
//!    joined: 1626900000,
//!    left: None,
//! };
//! // Put the new employee item into the db
//! client.put(new_employee).execute().await?;
//!
//! let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
//!    .name("John") // query the db for all employees named "John"
//!    .execute().await?;
//!
//! let employees: Vec<EmployeeNameIndex> = client.query::<EmployeeNameIndex>()
//!    .name("John") // query the db for all employees named "John"
//!    .joined().between(1626649200, 1626735600) // and who joined between 2021-07-19 and 2021-07-20
//!    .execute().await?;
//! ```

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![deny(missing_docs)]

use std::convert::TryFrom;

use attr::field;
use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, DeriveInput};

mod attr;
mod convert;
mod key;
mod query;
mod iter;

trait Builder {
    fn parse(vis: syn::Visibility, name: syn::Ident, generics: syn::Generics, attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<proc_macro2::TokenStream>;
}

fn derive<P: Builder>(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let DeriveInput { attrs, vis, ident, generics, data } = input;

    match data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(fields) => match P::parse(vis, ident, generics, attrs, fields) {
                Ok(t) => t,
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
    derive::<key::Builder>(input)
}

/// Implement a strongly typed query builder. This is used to setup query requests
#[proc_macro_derive(Query, attributes(nitro))]
pub fn derive_query(input: TokenStream) -> TokenStream {
    derive::<query::Builder>(input)
}

/// Implement `Into<Attributes>` and `TryFrom<Attributes>`
#[proc_macro_derive(Attributes, attributes(nitro))]
pub fn derive_convert(input: TokenStream) -> TokenStream {
    derive::<convert::Builder>(input)
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

#[derive(Clone, Copy)]
struct DL;
impl From<DL> for syn::Lifetime {
    fn from(_: DL) -> Self {
        parse_quote!('__nitroglycerin_dynamo_db_dlient)
    }
}
impl quote::ToTokens for DL {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        syn::Lifetime::from(*self).to_tokens(tokens);
    }
}

#[derive(Clone)]
struct NamedField {
    pub attrs: field::Attr,
    pub name: syn::Ident,
    pub ty: syn::Type,
}

impl TryFrom<syn::Field> for NamedField {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> syn::Result<Self> {
        let syn::Field { ident, attrs, ty, .. } = field;
        let attrs = field::Attr::parse_attrs(attrs)?;
        Ok(Self { attrs, name: ident.unwrap(), ty })
    }
}

#[derive(Clone)]
struct Column {
    pub ident: syn::Ident,
    pub name: String,
    pub ty: syn::Type,
    pub with: Option<syn::Path>,
}

impl From<NamedField> for Column {
    fn from(f: NamedField) -> Self {
        Self {
            name: f.attrs.rename.as_ref().map_or_else(|| f.name.to_string(), syn::LitStr::value),
            ident: f.name,
            ty: f.ty,
            with: f.attrs.with,
        }
    }
}
