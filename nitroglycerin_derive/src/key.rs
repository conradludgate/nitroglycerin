use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Generics, Ident, Type, Visibility};

use crate::{Column, NamedField, D, DL};

pub fn derive(vis: Visibility, name: syn::Ident, generics: syn::Generics, _attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<TokenStream> {
    let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;

    let partition_key: Column = fields
        .iter()
        .find_map(|f| f.attrs.partition_key.map(|_| f.clone().into()))
        .ok_or_else(|| syn::Error::new(Span::call_site(), "table needs a partition key"))?;

    let sort_key = fields.iter().find_map(|f| f.attrs.sort_key.map(|_| f.clone().into()));

    Ok(KeyBuilder::new(vis, name, generics, partition_key, sort_key).to_token_stream())
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

struct KeyBuilder {
    trait_builder: TraitBuilder,
    key_builder1: KeyBuilder1,
    key_builder2: KeyBuilder2,
}

impl KeyBuilder {
    pub fn new(vis: Visibility, output: Ident, generics: Generics, partition_key: Column, sort_key: Option<Column>) -> Self {
        Self {
            trait_builder: TraitBuilder::new(output.clone(), generics.clone()),
            key_builder1: KeyBuilder1::new(vis.clone(), output.clone(), generics.clone(), partition_key),
            key_builder2: KeyBuilder2::new(vis, output, generics, sort_key),
        }
    }
}

impl ToTokens for KeyBuilder {
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

struct TraitBuilder {
    output: Ident,
    generics: Generics,
    generics2: Generics,
}

impl TraitBuilder {
    fn new(output: Ident, mut generics: Generics) -> Self {
        let generics2 = generics.clone();

        let where_clause = generics.make_where_clause();
        where_clause.predicates.push(parse_quote! {
            Self: ::nitroglycerin::Table
        });
        where_clause.predicates.push(parse_quote! {
            #R: ::std::convert::From<::nitroglycerin::key::Key>
        });
        generics.params.push(parse_quote! { #DL });
        generics.params.push(parse_quote! { #D: #DL + ?Sized });
        generics.params.push(parse_quote! { #R });

        Self { output, generics, generics2 }
    }
}

impl ToTokens for TraitBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { output, generics, generics2 } = self;
        let builder = format_ident!("{}KeyBuilder", output);

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let (_, ty_generics2, _) = generics2.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::nitroglycerin::key::Builder<#DL, #D, #R> for #output #ty_generics2 #where_clause {
                type Builder = #builder #ty_generics;

                fn key(client: &#DL #D) -> Self::Builder {
                    Self::Builder { client, _phantom: ::std::marker::PhantomData }
                }
            }
        })
    }
}

struct KeyBuilder1 {
    vis: Visibility,
    output: Ident,
    generics: Generics,
    generics2: Generics,
    phantom_data: Type,
    partition_key: Column,
}

impl KeyBuilder1 {
    fn new(vis: Visibility, output: Ident, mut generics: Generics, partition_key: Column) -> Self {
        let generics2 = generics.clone();

        let tys = generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #R,
                #(
                    #tys,
                )*
            )
        };
        generics.params.push(parse_quote! { #DL });
        generics.params.push(parse_quote! { #D: #DL + ?Sized });
        generics.params.push(parse_quote! { #R });

        let where_clause = generics.make_where_clause();
        where_clause.predicates.push(parse_quote! {
            #R: ::std::convert::From<::nitroglycerin::key::Key>
        });

        Self {
            vis,
            output,
            generics,
            generics2,
            phantom_data,
            partition_key,
        }
    }
}

impl ToTokens for KeyBuilder1 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            generics,
            generics2,
            phantom_data,
            partition_key,
        } = self;
        let builder = format_ident!("{}KeyBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let Column {
            ident: p_ident,
            name: p_name,
            ty: p_ty,
        } = partition_key;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let (_, ty_generics2, _) = generics2.split_for_impl();

        tokens.extend(quote! {
            #vis struct #builder #impl_generics {
                client: &#DL #D,
                _phantom: ::std::marker::PhantomData<#phantom_data>,
            }

            impl #impl_generics #builder #ty_generics #where_clause {
                #vis fn #p_ident(self, #p_ident: impl ::std::convert::Into<#p_ty>) -> #builder_p #ty_generics
                where
                    #p_ty: ::nitroglycerin::convert::IntoAttributeValue,
                    #output #ty_generics2: ::nitroglycerin::Table,
                {
                    let partition_key: #p_ty = #p_ident.into();
                    let Self { client, _phantom } = self;

                    let key = ::nitroglycerin::key::Key::new::<#output #ty_generics2, _>(#p_name, partition_key);

                    #builder_p::new(client, key)
                }
            }
        })
    }
}

struct KeyBuilder2 {
    vis: Visibility,
    output: Ident,
    generics: Generics,
    generics2: Generics,
    phantom_data: Type,
    sort_key: Option<Column>,
}

impl KeyBuilder2 {
    fn new(vis: Visibility, output: Ident, mut generics: Generics, sort_key: Option<Column>) -> Self {
        let generics2 = generics.clone();

        let tys = generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #R,
                #(
                    #tys,
                )*
            )
        };
        generics.params.push(parse_quote! { #DL });
        generics.params.push(parse_quote! { #D: #DL + ?Sized });
        generics.params.push(parse_quote! { #R });

        let where_clause = generics.make_where_clause();
        where_clause.predicates.push(parse_quote! {
            #R: ::std::convert::From<::nitroglycerin::key::Key>
        });

        Self {
            vis,
            output,
            generics,
            generics2,
            phantom_data,
            sort_key,
        }
    }
}

impl ToTokens for KeyBuilder2 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            sort_key,
            generics,
            generics2,
            phantom_data,
        } = self;

        let builder = format_ident!("{}KeyBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let (_, ty_generics2, _) = generics2.split_for_impl();

        match sort_key {
            Some(Column {
                ident: s_ident,
                name: s_name,
                ty: s_ty,
            }) => tokens.extend(quote! {
                #vis struct #builder_p #impl_generics {
                    client: &#DL #D,
                    key: ::nitroglycerin::key::Key,
                    _phantom: ::std::marker::PhantomData<#phantom_data>,
                }

                impl #impl_generics #builder_p #ty_generics #where_clause {
                    fn new(client: &#DL #D, key: ::nitroglycerin::key::Key) -> Self {
                        Self { client, key, _phantom: ::std::marker::PhantomData }
                    }

                    #vis fn #s_ident(self, #s_ident: impl ::std::convert::Into<#s_ty>) -> ::nitroglycerin::key::Expr<#DL, #D, #R, #output #ty_generics2>
                    where
                        #s_ty: ::nitroglycerin::convert::IntoAttributeValue,
                    {
                        let sort_key: #s_ty = #s_ident.into();
                        let Self { client, mut key, _phantom } = self;

                        key.insert(#s_name, sort_key);

                        ::nitroglycerin::key::Expr::new(client, key)
                    }
                }
            }),
            None => tokens.extend(quote! {
                #vis type #builder_p #ty_generics = ::nitroglycerin::key::Expr<#DL, #D, #R, #output #ty_generics2>;
            }),
        }
    }
}
