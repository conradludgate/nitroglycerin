use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_quote, Generics, Ident, Type, Visibility};

use crate::{Column, NamedField, D, DL};

impl<'a> crate::Builder for Builder<'a> {
    fn parse(vis: Visibility, name: syn::Ident, generics: syn::Generics, _attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<TokenStream> {
        let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;
    
        let partition_key: Column = fields
            .iter()
            .find_map(|f| f.attrs.partition_key.map(|_| f.clone().into()))
            .ok_or_else(|| syn::Error::new(Span::call_site(), "table needs a partition key"))?;
    
        let sort_key = fields.iter().find_map(|f| f.attrs.sort_key.map(|_| f.clone().into()));
    
        Ok(Builder::new(&vis, &name, &generics, partition_key, sort_key).to_token_stream())
    }
}

pub struct Builder<'a> {
    trait_builder: TraitBuilder<'a>,
    query_builder1: Builder1<'a>,
    query_builder2: Builder2<'a>,
}

impl<'a> Builder<'a> {
    fn new(vis: &'a Visibility, output: &'a Ident, generics: &'a Generics, partition_key: Column, sort_key: Option<Column>) -> Self {
        Self {
            trait_builder: TraitBuilder::new(output, generics),
            query_builder1: Builder1::new(vis, output, generics, partition_key),
            query_builder2: Builder2::new(vis, output, generics, sort_key),
        }
    }
}

impl<'a> ToTokens for Builder<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            trait_builder,
            query_builder1,
            query_builder2,
        } = self;
        trait_builder.to_tokens(tokens);
        query_builder1.to_tokens(tokens);
        query_builder2.to_tokens(tokens);
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

        new_generics.make_where_clause().predicates.push(parse_quote! {
            Self:  ::nitroglycerin::TableIndex
        });
        new_generics.params.push(parse_quote! { #DL });
        new_generics.params.push(parse_quote! { #D: #DL + ?Sized });

        Self { output, generics, new_generics }
    }
}

impl<'a> ToTokens for TraitBuilder<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { output, generics, new_generics } = self;
        let builder = format_ident!("{}QueryBuilder", output);

        let (impl_generics, ty_generics, where_clause) = new_generics.split_for_impl();
        let (_, ty_generics2, _) = generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::nitroglycerin::query::Query<#DL, #D> for #output #ty_generics2 #where_clause {
                type Builder = #builder #ty_generics;

                fn query(client: &#DL #D) -> Self::Builder {
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
                #(
                    #tys,
                )*
            )
        };

        new_generics.params.push(parse_quote! { #DL });
        new_generics.params.push(parse_quote! { #D: #DL + ?Sized });

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
        let builder = format_ident!("{}QueryBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let Column { ident, name, ty, .. } = partition_key;

        let (impl_generics, ty_generics, where_clause) = new_generics.split_for_impl();
        let (_, ty_generics2, _) = generics.split_for_impl();

        let type_doc = format!("part one of the query builder chain for {}", output);

        tokens.extend(quote! {
            #[doc = #type_doc]
            #vis struct #builder #impl_generics {
                client: &#DL #D,
                _phantom: ::std::marker::PhantomData<#phantom_data>,
            }
        });

        let fn_doc = format!("set the value of the sort key ({})", ident);

        tokens.extend(quote_spanned! { ident.span() =>
            impl #impl_generics #builder #ty_generics #where_clause {
                #[doc = #fn_doc]
                #vis fn #ident<T>(self, #ident: &T) -> ::std::result::Result<#builder_p #ty_generics, ::nitroglycerin::ser::Error>
                where
                    #ty: ::std::borrow::Borrow<T>,
                    T: ::nitroglycerin::serde::Serialize + ?::std::marker::Sized,
                    #output #ty_generics2: ::nitroglycerin::TableIndex,
                {
                    let partition_key: &T = #ident;
                    let Self { client, _phantom } = self;

                    let input = ::nitroglycerin::query::new_input::<#output #ty_generics2, _>(#name, partition_key)?;

                    ::std::result::Result::Ok(#builder_p::new(client, input))
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
                #(
                    #tys,
                )*
            )
        };
        new_generics.params.push(parse_quote! { #DL });
        new_generics.params.push(parse_quote! { #D: #DL + ?Sized });

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

        let builder = format_ident!("{}QueryBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let (impl_generics, ty_generics, where_clause) = new_generics.split_for_impl();
        let (_, ty_generics2, _) = generics.split_for_impl();

        let type_doc = format!("part two of the query builder chain for {}", output);

        match sort_key {
            Some(Column { ident, name, ty, .. }) => {
                tokens.extend(quote! {
                    #[doc = #type_doc]
                    #vis struct #builder_p #impl_generics {
                        client: &#DL #D,
                        input: ::nitroglycerin::dynamodb::QueryInput,
                        _phantom: ::std::marker::PhantomData<#phantom_data>,
                    }

                    impl #impl_generics #builder_p #ty_generics #where_clause {
                        fn new(client: &#DL #D, input: ::nitroglycerin::dynamodb::QueryInput) -> Self {
                            Self { client, input, _phantom: ::std::marker::PhantomData }
                        }

                        #vis fn consistent_read(self) -> ::nitroglycerin::query::Expr<#DL, #D, #output #ty_generics2> {
                            let Self { client, input, _phantom } = self;
                            ::nitroglycerin::query::Expr::new(client, input).consistent_read()
                        }

                        #vis async fn execute(self) -> ::std::result::Result<::std::vec::Vec<#output #ty_generics2>, ::nitroglycerin::DynamoError<::nitroglycerin::dynamodb::QueryError>>
                        where
                            #D: ::nitroglycerin::dynamodb::DynamoDb,
                            &#DL #D: ::std::marker::Send,
                            #output #ty_generics2: ::nitroglycerin::serde::de::DeserializeOwned + ::std::marker::Send,
                        {
                            let Self { client, input, _phantom } = self;
                            ::nitroglycerin::query::Expr::new(client, input).execute().await
                        }
                    }
                });

                let fn_doc = format!("set the value of the sort key ({})", ident);

                tokens.extend(quote_spanned! { ident.span() =>
                    impl #impl_generics #builder_p #ty_generics #where_clause {
                        #[doc = #fn_doc]
                        #vis fn #ident(self) -> ::nitroglycerin::query::BuilderSort<#DL, #D, #ty, #output #ty_generics2> {
                            let Self { client, input, _phantom } = self;
                            ::nitroglycerin::query::BuilderSort::new(client, input, #name)
                        }
                    }
                });
            }
            None => tokens.extend(quote! {
                #[doc = #type_doc]
                #vis type #builder_p #ty_generics = ::nitroglycerin::query::Expr<#DL, #D, #output #ty_generics2>;
            }),
        }
    }
}
