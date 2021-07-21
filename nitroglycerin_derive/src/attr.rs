use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use syn::{parse2, Token};

use crate::iter::{Ext, SplitByState};

pub mod field;

fn equal<T: syn::parse::Parse>(tokens: TokenStream) -> syn::Result<T> {
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

    Ok(parse2::<Equal<_>>(tokens)?.t)
}

fn empty(tokens: TokenStream) -> syn::Result<()> {
    struct Empty {}
    impl syn::parse::Parse for Empty {
        fn parse(_: syn::parse::ParseStream) -> syn::Result<Self> {
            Ok(Self {})
        }
    }

    parse2::<Empty>(tokens)?;
    Ok(())
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
        for tt in tokens {
            let (inner, span) = match tt {
                TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Ok((g.stream(), g.span())),
                t => Err(syn::Error::new_spanned(t, "expected parenthesied attribute arguments")),
            }?;

            self.parse_args(span, inner)?;
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
                SplitByState::Split(p) => span = p.span(),
                SplitByState::Finished => break Ok(()),
                SplitByState::Continue => unreachable!(),
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

        self.parse(ident, tokens.collect())
    }

    fn parse(&mut self, ident: Ident, tokens: TokenStream) -> syn::Result<()>;
}
