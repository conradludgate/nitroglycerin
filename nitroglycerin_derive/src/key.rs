use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_quote, Generics, Ident, Type, Visibility};

use crate::{Column, D, DL, NamedField};

impl<'a> crate::Builder for Builder<'a> {
    fn parse(vis: syn::Visibility, name: syn::Ident, generics: syn::Generics, _attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<TokenStream> {
        let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;

        let partition_key: Column = fields
            .iter()
            .find_map(|f| f.attrs.partition_key.map(|_| f.clone().into()))
            .ok_or_else(|| syn::Error::new(Span::call_site(), "table needs a partition key"))?;

        let sort_key = fields.iter().find_map(|f| f.attrs.sort_key.map(|_| f.clone().into()));

        Ok(Builder::new(&vis, &name, &generics, partition_key, sort_key).into_token_stream())
    }
}

#[derive(Clone, Copy)]
struct R;
impl From<R> for syn::Ident {
    fn from(_: R) -> Self {
        parse_quote!(__NitroglycerinKeyRequest)
    }
}
impl quote::ToTokens for R {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        syn::Ident::from(*self).to_tokens(tokens);
    }
}

pub struct Builder<'a> {
    trait_builder: TraitBuilder<'a>,
    key_builder1: Builder1<'a>,
    key_builder2: Builder2<'a>,
}

impl<'a> Builder<'a> {
    fn new(vis: &'a Visibility, output: &'a Ident, generics: &'a Generics, partition_key: Column, sort_key: Option<Column>) -> Self {
        Self {
            trait_builder: TraitBuilder::new(output, generics),
            key_builder1: Builder1::new(vis, output, generics, partition_key),
            key_builder2: Builder2::new(vis, output, generics, sort_key),
        }
    }
}

impl<'a> ToTokens for Builder<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            trait_builder,
            key_builder1,
            key_builder2,
        } = self;
        trait_builder.to_tokens(tokens);
        key_builder1.to_tokens(tokens);
        key_builder2.to_tokens(tokens);
    }
}

struct TraitBuilder<'a> {
    output: &'a Ident,
    generics: &'a Generics,
    new_generics: Generics,
}

impl<'a> TraitBuilder<'a> {
    fn new(output: &'a Ident, generics: &'a Generics) -> Self {
        let mut new_generics = generics.clone();

        let where_clause = new_generics.make_where_clause();
        where_clause.predicates.push(parse_quote! {
            Self: ::nitroglycerin::Table
        });
        where_clause.predicates.push(parse_quote! {
            #R: ::std::convert::From<::nitroglycerin::key::Key>
        });
        new_generics.params.push(parse_quote! { #DL });
        new_generics.params.push(parse_quote! { #D: #DL + ?Sized });
        new_generics.params.push(parse_quote! { #R });

        Self { output, generics, new_generics }
    }
}

impl<'a> ToTokens for TraitBuilder<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { output, generics, new_generics } = self;
        let builder = format_ident!("{}KeyBuilder", output);

        let (impl_generics, ty_generics, where_clause) = new_generics.split_for_impl();
        let (_, ty_generics2, _) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::nitroglycerin::key::Builder<#DL, #D, #R> for #output #ty_generics2 #where_clause {
                type Builder = #builder #ty_generics;

                fn key(client: &#DL #D) -> Self::Builder {
                    Self::Builder { client, _phantom: ::std::marker::PhantomData }
                }
            }
        });
    }
}

struct Builder1<'a> {
    vis: &'a Visibility,
    output: &'a Ident,
    generics: &'a Generics,
    new_generics: Generics,
    phantom_data: Type,
    partition_key: Column,
}

impl<'a> Builder1<'a> {
    fn new(vis: &'a Visibility, output: &'a Ident, generics: &'a Generics, partition_key: Column) -> Self {
        let mut new_generics = generics.clone();

        let tys = new_generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #R,
                #(
                    #tys,
                )*
            )
        };
        new_generics.params.push(parse_quote! { #DL });
        new_generics.params.push(parse_quote! { #D: #DL + ?Sized });
        new_generics.params.push(parse_quote! { #R });

        let where_clause = new_generics.make_where_clause();
        where_clause.predicates.push(parse_quote! {
            #R: ::std::convert::From<::nitroglycerin::key::Key>
        });

        Self {
            vis,
            output,
            generics,
            new_generics,
            phantom_data,
            partition_key,
        }
    }
}

impl<'a> ToTokens for Builder1<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            generics,
            new_generics,
            phantom_data,
            partition_key,
        } = self;
        let builder = format_ident!("{}KeyBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let Column { ident, name, ty, .. } = partition_key;

        let (impl_generics, ty_generics, where_clause) = new_generics.split_for_impl();
        let (_, ty_generics2, _) = generics.split_for_impl();

        let type_doc = format!("part one of the key builder chain for {}", output);

        tokens.extend(quote! {
            #[doc = #type_doc]
            #vis struct #builder #impl_generics {
                client: &#DL #D,
                _phantom: ::std::marker::PhantomData<#phantom_data>,
            }
        });

        let fn_doc = format!("set the value of the partition key ({})", ident);

        tokens.extend(quote_spanned! { ident.span() =>
            impl #impl_generics #builder #ty_generics #where_clause {
                #[doc = #fn_doc]
                #vis fn #ident<T>(self, #ident: &T) -> ::std::result::Result<#builder_p #ty_generics, ::nitroglycerin::ser::Error>
                where
                    T: ::std::borrow::ToOwned<Owned = #ty> + ::nitroglycerin::serde::Serialize + ?::std::marker::Sized,
                    #output #ty_generics2: ::nitroglycerin::Table,
                {
                    let partition_key: &T = #ident;
                    let Self { client, _phantom } = self;

                    let key = ::nitroglycerin::key::Key::new::<#output #ty_generics2, _>(#name, partition_key)?;

                    ::std::result::Result::Ok(#builder_p::new(client, key))
                }
            }
        });
    }
}

struct Builder2<'a> {
    vis: &'a Visibility,
    output: &'a Ident,
    generics: &'a Generics,
    new_generics: Generics,
    phantom_data: Type,
    sort_key: Option<Column>,
}

impl<'a> Builder2<'a> {
    fn new(vis: &'a Visibility, output: &'a Ident, generics: &'a Generics, sort_key: Option<Column>) -> Self {
        let mut new_generics = generics.clone();

        let tys = new_generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #R,
                #(
                    #tys,
                )*
            )
        };
        new_generics.params.push(parse_quote! { #DL });
        new_generics.params.push(parse_quote! { #D: #DL + ?Sized });
        new_generics.params.push(parse_quote! { #R });

        let where_clause = new_generics.make_where_clause();
        where_clause.predicates.push(parse_quote! {
            #R: ::std::convert::From<::nitroglycerin::key::Key>
        });

        Self {
            vis,
            output,
            generics,
            new_generics,
            phantom_data,
            sort_key,
        }
    }
}

impl<'a> ToTokens for Builder2<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            sort_key,
            generics,
            new_generics,
            phantom_data,
        } = self;

        let builder = format_ident!("{}KeyBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let (impl_generics, ty_generics, where_clause) = new_generics.split_for_impl();
        let (_, ty_generics2, _) = generics.split_for_impl();

        let type_doc = format!("part two of the key builder chain for {}", output);

        match sort_key {
            Some(Column { ident, name, ty, .. }) => {
                tokens.extend(quote! {
                    #[doc = #type_doc]
                    #vis struct #builder_p #impl_generics {
                        client: &#DL #D,
                        key: ::nitroglycerin::key::Key,
                        _phantom: ::std::marker::PhantomData<#phantom_data>,
                    }

                    impl #impl_generics #builder_p #ty_generics #where_clause {
                        fn new(client: &#DL #D, key: ::nitroglycerin::key::Key) -> Self {
                            Self { client, key, _phantom: ::std::marker::PhantomData }
                        }
                    }
                });

                let fn_doc = format!("set the value of the sort key ({})", ident);

                tokens.extend(quote_spanned! { ident.span() =>
                    impl #impl_generics #builder_p #ty_generics #where_clause {
                        #[doc = #fn_doc]
                        #vis fn #ident<T>(self, #ident: &T) -> ::std::result::Result<::nitroglycerin::key::Expr<#DL, #D, #R, #output #ty_generics2>, ::nitroglycerin::ser::Error>
                        where
                            T: ::std::borrow::ToOwned<Owned = #ty> + ::nitroglycerin::serde::Serialize + ?::std::marker::Sized,
                        {
                            let sort_key: &T = #ident;
                            let Self { client, mut key, _phantom } = self;

                            key.insert(#name, sort_key)?;

                            ::std::result::Result::Ok(::nitroglycerin::key::Expr::new(client, key))
                        }
                    }
                });
            }
            None => tokens.extend(quote! {
                #[doc = #type_doc]
                #vis type #builder_p #ty_generics = ::nitroglycerin::key::Expr<#DL, #D, #R, #output #ty_generics2>;
            }),
        }
    }
}
