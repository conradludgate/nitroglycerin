use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Expr, Generics, Ident, Type, Visibility};

use crate::table::Column;

pub struct QueryBuilder {
    pub vis: Visibility,
    pub table_name: Expr,
    pub index_name: Option<Expr>,
    pub output: Ident,
    pub generics: Generics,

    pub partition_key: Column,
    pub sort_key: Option<Column>,
}

impl ToTokens for QueryBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            table_name,
            index_name,
            output,
            partition_key,
            sort_key,
            generics,
        } = self;

        let builder = format_ident!("{}QueryBuilder", output);

        let Column {
            ident: p_ident,
            name: p_name,
            ty: p_ty,
        } = partition_key;

        let builder_p = format_ident!("{}Partition", builder);

        let sort = match sort_key {
            Some(Column {
                ident: s_ident,
                name: s_name,
                ty: s_ty,
            }) => quote! {
                #vis struct #builder_p<D> {
                    client: D,
                    input: ::nitroglycerin::dynamodb::QueryInput,
                }

                impl<D> #builder_p<D> {
                    fn new(client: D, input: ::nitroglycerin::dynamodb::QueryInput) -> Self {
                        Self { client, input }
                    }

                    #vis fn #s_ident(self) -> ::nitroglycerin::query::QueryBuilderSort<D, #s_ty, #output> {
                        let Self { client, input } = self;
                        ::nitroglycerin::query::QueryBuilderSort::new(client, input, #s_name)
                    }

                    #vis fn consistent_read(self) -> ::nitroglycerin::query::QueryExpr<D, #output> {
                        let Self { client, input } = self;
                        ::nitroglycerin::query::QueryExpr::new(client, input).consistent_read()
                    }
                }

                impl<D: ::nitroglycerin::dynamodb::DynamoDb> #builder_p<D> {
                    #vis async fn execute(self) -> ::std::result::Result<::std::vec::Vec<#output>, ::nitroglycerin::DynamoError<::nitroglycerin::dynamodb::QueryError>> {
                        let Self { client, input } = self;
                        ::nitroglycerin::query::QueryExpr::new(client, input).execute().await
                    }
                }
            },
            None => quote! {
                #vis type #builder_p<D> = ::nitroglycerin::query::QueryExpr<D, #output>;
            },
        };

        let input = match index_name {
            Some(index_name) => quote! {
                let mut input = ::nitroglycerin::query::new_input(#table_name.into(), #p_name, partition_key);
                input.index_name = Some(#index_name.into());
            },
            None => quote! {
                let input = ::nitroglycerin::query::new_input(#table_name.into(), #p_name, partition_key);
            },
        };

        tokens.extend(quote! {
            impl<D> ::nitroglycerin::Query<D> for #output {
                type Builder = #builder<D>;

                fn query(client: D) -> Self::Builder {
                    Self::Builder { client }
                }
            }

            #vis struct #builder<D> {
                client: D,
            }

            impl<D> #builder<D> {
                #vis fn #p_ident(self, #p_ident: #p_ty) -> #builder_p<D> {
                    let partition_key = #p_ident;
                    let Self { client } = self;

                    #input

                    #builder_p::new(client, input)
                }
            }

            #sort
        })
    }
}
