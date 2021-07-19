use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use syn::{Expr, LitStr};

use crate::split_by::IterExt;

#[derive(Builder)]
pub struct TableAttr {
    pub table_name: Expr,
}

impl TableAttr {
    pub fn parse_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut output = TableAttrBuilder::default();
        for attr in attrs {
            if attr.path.is_ident("nitro") {
                output.parse_attr(attr.tokens)?;
            }
        }
        output.build().map_err(|err| syn::Error::new(Span::call_site(), err))
    }
}

impl TableAttrBuilder {
    pub fn parse_attr(&mut self, tokens: TokenStream) -> syn::Result<()> {
        for tt in tokens.into_iter() {
            let (inner, span) = match tt {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Ok((g.stream(), g.span())),
                t => Err(syn::Error::new_spanned(t, "expected parenthesied attribute arguments")),
            }?;

            self.parse_args(span, inner.into_iter().peekable())?;
        }
        Ok(())
    }

    pub fn parse_args(&mut self, mut span: Span, tokens: impl IntoIterator<Item = TokenTree>) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let mut split = tokens.split_by(|tt| match tt {
            TokenTree::Punct(p) => p.as_char() == ',',
            _ => false,
        });
        loop {
            self.parse_arg(span, &mut split)?;
            match split.done() {
                Some(Some(p)) => span = p.span(),
                Some(None) => break Ok(()),
                None => unreachable!(),
            }
        }
    }

    pub fn parse_arg(&mut self, span: Span, tokens: impl IntoIterator<Item = TokenTree>) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(t) => return Err(syn::Error::new_spanned(t, "expected ident")),
            None => return Err(syn::Error::new(span, "expected ident to follow")),
        };

        match ident.to_string().as_ref() {
            "table_name" => {
                self.table_name(equal(ident.span(), tokens)?);
            }
            _ => return Err(syn::Error::new_spanned(ident, "unknown parameter")),
        }
        Ok(())
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(derive(Debug))]
#[builder(setter(strip_option))]
pub struct FieldAttr {
    #[builder(default)]
    pub rename: Option<LitStr>,

    #[builder(default)]
    pub partition_key: Option<()>,

    #[builder(default)]
    pub sort_key: Option<()>,
}

impl FieldAttr {
    pub fn parse_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        let mut output = FieldAttrBuilder::default();
        for attr in attrs {
            if attr.path.is_ident("nitro") {
                output.parse_attr(attr.tokens)?;
            }
        }
        output.build().map_err(|err| syn::Error::new(Span::call_site(), err))
    }
}

impl FieldAttrBuilder {
    pub fn parse_attr(&mut self, tokens: TokenStream) -> syn::Result<()> {
        for tt in tokens.into_iter() {
            let (inner, span) = match tt {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Ok((g.stream(), g.span())),
                t => Err(syn::Error::new_spanned(t, "expected parenthesied attribute arguments")),
            }?;

            self.parse_args(span, inner.into_iter().peekable())?;
        }
        Ok(())
    }

    pub fn parse_args(&mut self, mut span: Span, tokens: impl IntoIterator<Item = TokenTree>) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let mut split = tokens.split_by(|tt| match tt {
            TokenTree::Punct(p) => p.as_char() == ',',
            _ => false,
        });
        loop {
            self.parse_arg(span, &mut split)?;
            match split.done() {
                Some(Some(p)) => span = p.span(),
                Some(None) => break Ok(()),
                None => unreachable!(),
            }
        }
    }

    pub fn parse_arg(&mut self, span: Span, tokens: impl IntoIterator<Item = TokenTree>) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(t) => return Err(syn::Error::new_spanned(t, "expected ident")),
            None => return Err(syn::Error::new(span, "expected ident to follow")),
        };

        match ident.to_string().as_ref() {
            "rename" => {
                self.rename(equal(ident.span(), tokens)?);
            }
            "partition_key" => {
                self.partition_key(empty(tokens)?);
            }
            "sort_key" => {
                self.sort_key(empty(tokens)?);
            }
            _ => return Err(syn::Error::new_spanned(ident, "unknown parameter")),
        }
        Ok(())
    }
}

fn equal<T: syn::parse::Parse>(span: Span, mut tokens: impl Iterator<Item = TokenTree>) -> syn::Result<T> {
    match tokens.next() {
        Some(TokenTree::Punct(p)) => {
            if p.as_char() != '=' {
                return Err(syn::Error::new_spanned(p, "expected an '=' to follow"));
            }
        }
        Some(t) => return Err(syn::Error::new_spanned(t, "expected an '=' to follow")),
        None => return Err(syn::Error::new(span, "expected an '=' to follow")),
    }

    syn::parse2(tokens.collect())
}

fn empty(tokens: impl Iterator<Item = TokenTree>) -> syn::Result<()> {
    let stream: TokenStream = tokens.collect();
    if stream.is_empty() {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(stream, "unexpected tokens"))
    }
}
