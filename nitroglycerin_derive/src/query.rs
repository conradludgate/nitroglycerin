use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Expr, Generics, Ident, Type, Visibility};

use crate::{table::Column, D};

pub struct QueryBuilder {
    trait_builder: TraitBuilder,
    query_builder1: QueryBuilder1,
    query_builder2: QueryBuilder2,
}

impl QueryBuilder {
    pub fn new(vis: Visibility, table_name: Expr, index_name: Option<Expr>, output: Ident, generics: Generics, partition_key: Column, sort_key: Option<Column>) -> Self {
        Self {
            trait_builder: TraitBuilder::new(output.clone(), generics.clone()),
            query_builder1: QueryBuilder1::new(vis.clone(), table_name, index_name, output.clone(), generics.clone(), partition_key),
            query_builder2: QueryBuilder2::new(vis, output, generics, sort_key),
        }
    }
}

impl ToTokens for QueryBuilder {
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

struct TraitBuilder {
    output: Ident,
    generics: Generics,
    generics2: Generics,
}

impl TraitBuilder {
    fn new(output: Ident, mut generics: Generics) -> Self {
        let generics2 = generics.clone();

        generics.make_where_clause().predicates.push(parse_quote! {
            Self: ::std::convert::TryFrom<
                ::std::collections::HashMap<String, ::nitroglycerin::dynamodb::AttributeValue>,
                Error = ::nitroglycerin::AttributeError,
            >
        });
        generics.params.push(parse_quote! { #D });

        Self { output, generics, generics2 }
    }
}

impl ToTokens for TraitBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { output, generics, generics2 } = self;
        let builder = format_ident!("{}QueryBuilder", output);

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let (_, ty_generics2, _) = generics2.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::nitroglycerin::Query<#D> for #output #ty_generics2 #where_clause {
                type Builder = #builder #ty_generics;

                fn query(client: #D) -> Self::Builder {
                    Self::Builder { client, _phantom: ::std::marker::PhantomData }
                }
            }
        })
    }
}

struct QueryBuilder1 {
    vis: Visibility,
    table_name: Expr,
    index_name: Option<Expr>,
    output: Ident,
    generics: Generics,
    phantom_data: Type,
    partition_key: Column,
}

impl QueryBuilder1 {
    fn new(vis: Visibility, table_name: Expr, index_name: Option<Expr>, output: Ident, mut generics: Generics, partition_key: Column) -> Self {
        let tys = generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #(
                    #tys,
                )*
            )
        };

        generics.params.push(parse_quote! { #D });

        Self {
            vis,
            table_name,
            index_name,
            output,
            generics,
            phantom_data,
            partition_key,
        }
    }
}

impl ToTokens for QueryBuilder1 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            table_name,
            index_name,
            output,
            generics,
            phantom_data,
            partition_key,
        } = self;
        let builder = format_ident!("{}QueryBuilder", output);
        let builder_p = format_ident!("{}Partition", builder);

        let Column {
            ident: p_ident,
            name: p_name,
            ty: p_ty,
        } = partition_key;

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let index = index_name.as_ref().map(|index_name| {
            quote! {
                let input = {
                    let mut input = input;
                    input.index_name = Some(#index_name.into());
                    input
                };
            }
        });

        tokens.extend(quote! {
            #vis struct #builder #impl_generics {
                client: #D,
                _phantom: ::std::marker::PhantomData<#phantom_data>,
            }

            impl #impl_generics #builder #ty_generics #where_clause {
                #vis fn #p_ident(self, #p_ident: #p_ty) -> #builder_p #ty_generics
                where
                    #p_ty: ::nitroglycerin::convert::IntoAttributeValue,
                {
                    let partition_key = #p_ident;
                    let Self { client, _phantom } = self;

                    let input = ::nitroglycerin::query::new_input(#table_name.into(), #p_name, partition_key);
                    #index

                    #builder_p::new(client, input)
                }
            }
        })
    }
}

struct QueryBuilder2 {
    vis: Visibility,
    output: Ident,
    generics: Generics,
    generics2: Generics,
    phantom_data: Type,
    sort_key: Option<Column>,
}

impl QueryBuilder2 {
    fn new(vis: Visibility, output: Ident, mut generics: Generics, sort_key: Option<Column>) -> Self {
        let generics2 = generics.clone();

        let tys = generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #(
                    #tys,
                )*
            )
        };
        generics.params.push(parse_quote! { #D });

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

impl ToTokens for QueryBuilder2 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            sort_key,
            generics,
            generics2,
            phantom_data,
        } = self;

        let builder = format_ident!("{}QueryBuilder", output);
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
                    client: #D,
                    input: ::nitroglycerin::dynamodb::QueryInput,
                    _phantom: ::std::marker::PhantomData<#phantom_data>,
                }

                impl #impl_generics #builder_p #ty_generics #where_clause {
                    fn new(client: #D, input: ::nitroglycerin::dynamodb::QueryInput) -> Self {
                        Self { client, input, _phantom: ::std::marker::PhantomData }
                    }

                    #vis fn #s_ident(self) -> ::nitroglycerin::query::QueryBuilderSort<#D, #s_ty, #output #ty_generics2> {
                        let Self { client, input, _phantom } = self;
                        ::nitroglycerin::query::QueryBuilderSort::new(client, input, #s_name)
                    }

                    #vis fn consistent_read(self) -> ::nitroglycerin::query::QueryExpr<#D, #output #ty_generics2> {
                        let Self { client, input, _phantom } = self;
                        ::nitroglycerin::query::QueryExpr::new(client, input).consistent_read()
                    }

                    #vis async fn execute(self) -> ::std::result::Result<::std::vec::Vec<#output #ty_generics2>, ::nitroglycerin::DynamoError<::nitroglycerin::dynamodb::QueryError>>
                    where
                        #D: ::nitroglycerin::dynamodb::DynamoDb,
                        #output #ty_generics2: ::std::convert::TryFrom<
                            ::std::collections::HashMap<String, ::nitroglycerin::dynamodb::AttributeValue>,
                            Error = ::nitroglycerin::AttributeError,
                        >,
                    {
                        let Self { client, input, _phantom } = self;
                        ::nitroglycerin::query::QueryExpr::new(client, input).execute().await
                    }
                }
            }),
            None => tokens.extend(quote! {
                #vis type #builder_p #impl_generics = ::nitroglycerin::query::QueryExpr<#D, #output #ty_generics2>;
            }),
        }
    }
}
