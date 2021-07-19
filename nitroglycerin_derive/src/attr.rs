use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use syn::{parse2, Expr, LitStr, Token};

use crate::split_by::IterExt;

#[derive(Builder)]
pub struct TableAttr {
    pub table_name: Expr,
}

impl TableAttr {
    pub fn parse_attrs(attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        TableAttrBuilder::default().parse_attrs(attrs)?.build().map_err(|err| syn::Error::new(Span::call_site(), err))
    }
}

impl AttrBuilder for TableAttrBuilder {
    fn parse(&mut self, ident: Ident, tokens: impl Iterator<Item = TokenTree>) -> syn::Result<()> {
        match ident.to_string().as_ref() {
            "table_name" => {
                self.table_name(parse2::<Equal<_>>(tokens.collect())?.t);
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
        FieldAttrBuilder::default().parse_attrs(attrs)?.build().map_err(|err| syn::Error::new(Span::call_site(), err))
    }
}

impl AttrBuilder for FieldAttrBuilder {
    fn parse(&mut self, ident: Ident, tokens: impl Iterator<Item = TokenTree>) -> syn::Result<()> {
        match ident.to_string().as_ref() {
            "rename" => {
                self.rename(parse2::<Equal<_>>(tokens.collect())?.t);
            }
            "partition_key" => {
                self.partition_key(());
            }
            "sort_key" => {
                self.sort_key(());
            }
            _ => return Err(syn::Error::new_spanned(ident, "unknown parameter")),
        }
        Ok(())
    }
}

struct Equal<T> {
    _equal: Token![=],
    t: T,
}

impl<T: syn::parse::Parse> syn::parse::Parse for Equal<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _equal: input.parse()?,
            t: input.parse()?,
        })
    }
}

trait AttrBuilder: Sized {
    fn parse_attrs(mut self, attrs: Vec<syn::Attribute>) -> syn::Result<Self> {
        for attr in attrs {
            if attr.path.is_ident("nitro") {
                self.parse_attr(attr.tokens)?;
            }
        }
        Ok(self)
    }

    fn parse_attr(&mut self, tokens: TokenStream) -> syn::Result<()> {
        for tt in tokens.into_iter() {
            let (inner, span) = match tt {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Ok((g.stream(), g.span())),
                t => Err(syn::Error::new_spanned(t, "expected parenthesied attribute arguments")),
            }?;

            self.parse_args(span, inner.into_iter().peekable())?;
        }
        Ok(())
    }

    fn parse_args(&mut self, mut span: Span, tokens: impl IntoIterator<Item = TokenTree>) -> syn::Result<()> {
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

    fn parse_arg(&mut self, span: Span, tokens: impl IntoIterator<Item = TokenTree>) -> syn::Result<()> {
        let mut tokens = tokens.into_iter();
        let ident = match tokens.next() {
            Some(TokenTree::Ident(i)) => i,
            Some(t) => return Err(syn::Error::new_spanned(t, "expected ident")),
            None => return Err(syn::Error::new(span, "expected ident to follow")),
        };

        self.parse(ident, &mut tokens)?;

        let stream: TokenStream = tokens.collect();
        if stream.is_empty() {
            Ok(())
        } else {
            Err(syn::Error::new_spanned(stream, "unexpected tokens"))
        }
    }

    fn parse(&mut self, ident: Ident, tokens: impl Iterator<Item = TokenTree>) -> syn::Result<()>;
}
