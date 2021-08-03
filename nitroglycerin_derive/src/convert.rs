use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_quote, Generics, Ident};

use crate::{Column, NamedField};

impl<'a> crate::Builder for Builder<'a> {
    fn parse(_vis: syn::Visibility, name: syn::Ident, generics: syn::Generics, _attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<TokenStream> {
        let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;
        let columns: Vec<_> = fields.into_iter().map(Column::from).collect();
        Ok(Builder::new(&name, &generics, &columns).to_token_stream())
    }
}

pub struct Builder<'a> {
    from: FromBuilder<'a>,
    into: IntoBuilder<'a>,
}

impl<'a> Builder<'a> {
    fn new(ident: &'a Ident, generics: &'a Generics, columns: &'a [Column]) -> Self {
        Self {
            from: FromBuilder::new(ident, generics, columns),
            into: IntoBuilder::new(ident, generics, columns),
        }
    }
}

impl<'a> ToTokens for Builder<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { from, into } = self;
        from.to_tokens(tokens);
        into.to_tokens(tokens);
    }
}

struct IntoBuilder<'a> {
    pub ident: &'a Ident,
    pub generics: Generics,
    pub columns: &'a [Column],
}

impl<'a> IntoBuilder<'a> {
    fn new(ident: &'a Ident, generics: &'a Generics, columns: &'a [Column]) -> Self {
        let mut generics = generics.clone();
        let where_clause = generics.make_where_clause();

        for column in columns.iter().filter(|c| c.with.is_none()) {
            let ty = &column.ty;
            where_clause.predicates.push(parse_quote! {
                #ty: ::nitroglycerin::convert::IntoAttributeValue
            });
        }

        Self { ident, generics, columns }
    }
}

impl<'a> ToTokens for IntoBuilder<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, generics, columns } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let intos = columns.iter().map(|c| {
            let Column { ident, name, ty, with } = c;
            match with {
                None => quote_spanned! { ident.span() => (#name.to_owned(), <#ty as ::nitroglycerin::convert::IntoAttributeValue>::into_av(t.#ident)) },
                Some(with) => quote_spanned! { ident.span() => (#name.to_owned(), #with::into_av(t.#ident)) },
            }
        });

        tokens.extend(quote! {
            impl #impl_generics ::std::convert::From<#ident #ty_generics> for ::nitroglycerin::Attributes #where_clause {
                fn from(t: #ident #ty_generics) -> Self {
                    <_>::into_iter([
                        #( #intos ),*
                    ]).collect()
                }
            }

            impl #impl_generics ::nitroglycerin::convert::IntoAttributeValue for #ident #ty_generics #where_clause
            {
                fn into_av(self) -> ::nitroglycerin::dynamodb::AttributeValue {
                    ::nitroglycerin::dynamodb::AttributeValue {
                        m: ::std::option::Option::Some(<Self as ::std::convert::Into<::nitroglycerin::Attributes>>::into(self)),
                        ..::nitroglycerin::dynamodb::AttributeValue::default()
                    }
                }
            }
        });
    }
}

struct FromBuilder<'a> {
    pub ident: &'a Ident,
    pub generics: Generics,
    pub columns: &'a [Column],
}

impl<'a> FromBuilder<'a> {
    fn new(ident: &'a Ident, generics: &'a Generics, columns: &'a [Column]) -> Self {
        let mut generics = generics.clone();
        let where_clause = generics.make_where_clause();

        for column in columns.iter().filter(|c| c.with.is_none()) {
            let ty = &column.ty;
            where_clause.predicates.push(parse_quote! {
                #ty: ::nitroglycerin::convert::FromAttributeValue
            });
        }

        Self { ident, generics, columns }
    }
}

impl<'a> ToTokens for FromBuilder<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, generics, columns } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let extracts = columns.iter().map(|c| {
            let Column { ident, name, ty, with } = c;
            match with {
                None => quote_spanned! { ident.span() => #ident: ::nitroglycerin::convert::extract::<#ty>(&mut a, #name)? },
                Some(with) => quote_spanned! { ident.span() => #ident: #with::try_from_av(a.remove(#name).ok_or_else(|| ::nitroglycerin::AttributeError::MissingField(#name.to_owned()))?)? },
            }
        });

        tokens.extend(quote! {
            impl #impl_generics ::std::convert::TryFrom<::nitroglycerin::Attributes> for #ident #ty_generics #where_clause {
                type Error = ::nitroglycerin::AttributeError;
                fn try_from(mut a: ::nitroglycerin::Attributes) -> ::std::result::Result<Self, Self::Error> {
                    Ok(Self { #( #extracts ),* })
                }
            }
        });

        tokens.extend(quote! {
            impl #impl_generics ::nitroglycerin::convert::FromAttributeValue for #ident #ty_generics #where_clause {
                fn try_from_av(av: ::nitroglycerin::dynamodb::AttributeValue) -> ::std::result::Result<Self, ::nitroglycerin::AttributeError> {
                    av.m.ok_or(::nitroglycerin::AttributeError::IncorrectType).and_then(
                        <Self as ::std::convert::TryFrom<::nitroglycerin::Attributes>>::try_from
                    )
                }
            }
        });
    }
}
