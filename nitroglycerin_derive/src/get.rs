use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Visibility};

use crate::query::Column;

pub struct GetBuilder {
    pub vis: Visibility,
    pub table_name: String,
    pub output: Ident,

    pub partition_key: Column,
    pub sort_key: Option<Column>,
}

impl ToTokens for GetBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            table_name,
            output,
            partition_key,
            sort_key,
        } = self;

        let builder = format_ident!("{}GetBuilder", output);

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
                    input: ::nitroglycerin::dynomite::dynamodb::GetItemInput,
                }

                impl<D> #builder_p<D> {
                    pub fn new(client: D, input: ::nitroglycerin::dynomite::dynamodb::GetItemInput) -> Self {
                        Self { client, input }
                    }

                    #vis fn #s_ident(self, #s_ident: #s_ty) -> ::nitroglycerin::get::GetExpr<D, #output> {
                        let sort_key = #s_ident;
                        let Self { client, mut input } = self;

                        input.key.insert(#s_name.to_owned(), sort_key.into_attr());

                        ::nitroglycerin::get::GetExpr::new(client, input)
                    }
                }
            },
            None => quote! {
                #vis type #builder_p<D> = ::nitroglycerin::get::GetExpr<D, #output>;
            },
        };

        tokens.extend(quote! {
            impl<D> ::nitroglycerin::Get<D> for #output {
                type Builder = #builder<D>;

                fn get(client: D) -> Self::Builder {
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

                    let input = ::nitroglycerin::get::new_input(#table_name, #p_name, partition_key);

                    #builder_p::new(client, input)
                }
            }

            #sort
        })
    }
}
