use std::convert::{Infallible, TryFrom, TryInto};

use proc_macro2::{Span, TokenStream};

use super::{empty, equal, AttrBuilder};

#[derive(Clone)]
pub struct Attr {
    pub rename: Option<syn::LitStr>,
    pub partition_key: Option<()>,
    pub sort_key: Option<()>,
}

impl Attr {
    pub fn parse_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        Builder::default().parse_attrs(attrs)?.try_into().map_err(|err| syn::Error::new(Span::call_site(), err))
    }
}

#[derive(Default)]
struct Builder {
    rename: Option<syn::LitStr>,
    partition_key: Option<()>,
    sort_key: Option<()>,
}

impl Builder {
    fn rename(&mut self, rename: syn::LitStr) -> &mut Self {
        self.rename = Some(rename);
        self
    }
    fn partition_key(&mut self, partition_key: ()) -> &mut Self {
        self.partition_key = Some(partition_key);
        self
    }
    fn sort_key(&mut self, sort_key: ()) -> &mut Self {
        self.sort_key = Some(sort_key);
        self
    }
}

impl TryFrom<Builder> for Attr {
    type Error = Infallible;
    fn try_from(value: Builder) -> Result<Self, Self::Error> {
        let Builder { rename, partition_key, sort_key } = value;
        Ok(Self { rename, partition_key, sort_key })
    }
}

impl AttrBuilder for Builder {
    fn parse(&mut self, ident: syn::Ident, tokens: TokenStream) -> syn::Result<()> {
        match ident.to_string().as_ref() {
            "rename" => self.rename(equal(tokens)?),
            "partition_key" => self.partition_key(empty(tokens)?),
            "sort_key" => self.sort_key(empty(tokens)?),
            _ => return Err(syn::Error::new_spanned(ident, "unknown parameter")),
        };
        Ok(())
    }
}
