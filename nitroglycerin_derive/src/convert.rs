use std::convert::TryFrom;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Generics, Ident};

use crate::{Column, NamedField};

pub fn derive(_vis: syn::Visibility, name: syn::Ident, generics: syn::Generics, _attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<TokenStream> {
    let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;
    let columns: Vec<_> = fields.into_iter().map(Column::from).collect();
    Ok(ConvertBuilder::new(name, generics, columns).to_token_stream())
}

struct ConvertBuilder {
    from: FromBuilder,
    into: IntoBuilder,
}

impl ConvertBuilder {
    fn new(ident: Ident, generics: Generics, columns: Vec<Column>) -> Self {
        Self {
            from: FromBuilder::new(ident.clone(), generics.clone(), columns.clone()),
            into: IntoBuilder::new(ident, generics, columns),
        }
    }
}

impl ToTokens for ConvertBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { from, into } = self;
        from.to_tokens(tokens);
        into.to_tokens(tokens);
    }
}

struct IntoBuilder {
    pub ident: Ident,
    pub generics: Generics,
    pub columns: Vec<Column>,
}

impl IntoBuilder {
    fn new(ident: Ident, mut generics: Generics, columns: Vec<Column>) -> Self {
        let where_clause = generics.make_where_clause();

        for column in &columns {
            let ty = &column.ty;
            where_clause.predicates.push(parse_quote! {
                #ty: ::nitroglycerin::convert::IntoAttributeValue
            });
        }

        Self { ident, generics, columns }
    }
}

impl ToTokens for IntoBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, generics, columns } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let idents = columns.iter().map(|c| &c.ident);
        let names = columns.iter().map(|c| &c.name);
        let tys = columns.iter().map(|c| &c.ty);

        tokens.extend(quote! {
            impl #impl_generics ::std::convert::From<#ident #ty_generics> for ::nitroglycerin::Attributes #where_clause {
                fn from(t: #ident #ty_generics) -> Self {
                    <_>::into_iter([
                        #(
                            (#names.to_owned(), <#tys as ::nitroglycerin::convert::IntoAttributeValue>::into_av(t.#idents)),
                        )*
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

struct FromBuilder {
    pub ident: Ident,
    pub generics: Generics,
    pub columns: Vec<Column>,
}

impl FromBuilder {
    fn new(ident: Ident, mut generics: Generics, columns: Vec<Column>) -> Self {
        let where_clause = generics.make_where_clause();

        for column in &columns {
            let ty = &column.ty;
            where_clause.predicates.push(parse_quote! {
                #ty: ::nitroglycerin::convert::FromAttributeValue
            });
        }

        Self { ident, generics, columns }
    }
}

impl ToTokens for FromBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, generics, columns } = self;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let idents = columns.iter().map(|c| &c.ident);
        let names = columns.iter().map(|c| &c.name);
        let tys = columns.iter().map(|c| &c.ty);

        tokens.extend(quote! {
            impl #impl_generics ::std::convert::TryFrom<::nitroglycerin::Attributes> for #ident #ty_generics #where_clause {
                type Error = ::nitroglycerin::AttributeError;
                fn try_from(mut a: ::nitroglycerin::Attributes) -> ::std::result::Result<Self, Self::Error> {
                    Ok(Self {
                        #(
                            #idents: ::nitroglycerin::convert::extract::<#tys>(&mut a, #names)?,
                        )*
                    })
                }
            }

            impl #impl_generics ::nitroglycerin::convert::FromAttributeValue for #ident #ty_generics #where_clause {
                fn try_from_av(av: ::nitroglycerin::dynamodb::AttributeValue) -> ::std::result::Result<Self, ::nitroglycerin::AttributeError> {
                    av.m.ok_or(::nitroglycerin::AttributeError::IncorrectType).and_then(
                        <Self as ::std::convert::TryFrom<::nitroglycerin::Attributes, Error = ::nitroglycerin::AttributeError>>::try_from
                    )
                }
            }
        });
    }
}
