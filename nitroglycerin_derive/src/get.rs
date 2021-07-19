use std::convert::TryFrom;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, Generics, Ident, Type, Visibility};

use crate::{Column, NamedField, D};

pub fn derive(vis: Visibility, name: syn::Ident, generics: syn::Generics, _attrs: Vec<syn::Attribute>, fields: syn::FieldsNamed) -> syn::Result<TokenStream> {
    let fields: Vec<_> = fields.named.into_iter().map(NamedField::try_from).collect::<syn::Result<_>>()?;

    let partition_key: Column = fields
        .iter()
        .find_map(|f| f.attrs.partition_key.map(|_| f.clone().into()))
        .ok_or_else(|| syn::Error::new(Span::call_site(), "table needs a partition key"))?;

    let sort_key = fields.iter().find_map(|f| f.attrs.sort_key.map(|_| f.clone().into()));

    Ok(GetBuilder::new(vis, name, generics, partition_key, sort_key).to_token_stream())
}

struct GetBuilder {
    trait_builder: TraitBuilder,
    get_builder1: GetBuilder1,
    get_builder2: GetBuilder2,
}

impl GetBuilder {
    pub fn new(vis: Visibility, output: Ident, generics: Generics, partition_key: Column, sort_key: Option<Column>) -> Self {
        Self {
            trait_builder: TraitBuilder::new(output.clone(), generics.clone()),
            get_builder1: GetBuilder1::new(vis.clone(), output.clone(), generics.clone(), partition_key),
            get_builder2: GetBuilder2::new(vis, output, generics, sort_key),
        }
    }
}

impl ToTokens for GetBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            trait_builder,
            get_builder1,
            get_builder2,
        } = self;
        trait_builder.to_tokens(tokens);
        get_builder1.to_tokens(tokens);
        get_builder2.to_tokens(tokens);
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
            Self: ::nitroglycerin::Table
        });
        generics.params.push(parse_quote! { #D });

        Self { output, generics, generics2 }
    }
}

impl ToTokens for TraitBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { output, generics, generics2 } = self;
        let builder = format_ident!("{}GetBuilder", output);

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let (_, ty_generics2, _) = generics2.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics ::nitroglycerin::get::Get<#D> for #output #ty_generics2 #where_clause {
                type Builder = #builder #ty_generics;

                fn get(client: #D) -> Self::Builder {
                    Self::Builder { client, _phantom: ::std::marker::PhantomData }
                }
            }
        })
    }
}

struct GetBuilder1 {
    vis: Visibility,
    output: Ident,
    generics: Generics,
    generics2: Generics,
    phantom_data: Type,
    partition_key: Column,
}

impl GetBuilder1 {
    fn new(vis: Visibility, output: Ident, mut generics: Generics, partition_key: Column) -> Self {
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
            partition_key,
        }
    }
}

impl ToTokens for GetBuilder1 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            generics,
            generics2,
            phantom_data,
            partition_key,
        } = self;
        let builder = format_ident!("{}GetBuilder", output);
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
                client: #D,
                _phantom: ::std::marker::PhantomData<#phantom_data>,
            }

            impl #impl_generics #builder #ty_generics #where_clause {
                #vis fn #p_ident(self, #p_ident: #p_ty) -> #builder_p #ty_generics
                where
                    #p_ty: ::nitroglycerin::convert::IntoAttributeValue,
                    #output #ty_generics2: ::nitroglycerin::Table,
                {
                    let partition_key = #p_ident;
                    let Self { client, _phantom } = self;

                    let input = ::nitroglycerin::get::new_input::<#output #ty_generics2, _>(#p_name, partition_key);

                    #builder_p::new(client, input)
                }
            }
        })
    }
}

struct GetBuilder2 {
    vis: Visibility,
    output: Ident,
    generics: Generics,
    generics2: Generics,
    phantom_data: Type,
    sort_key: Option<Column>,
}

impl GetBuilder2 {
    fn new(vis: Visibility, output: Ident, mut generics: Generics, sort_key: Option<Column>) -> Self {
        let tys = generics.type_params().map(|tp| &tp.ident);
        let phantom_data = parse_quote! {
            (
                #(
                    #tys,
                )*
            )
        };

        let generics2 = generics.clone();
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

impl ToTokens for GetBuilder2 {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            vis,
            output,
            sort_key,
            generics,
            generics2,
            phantom_data,
        } = self;

        let builder = format_ident!("{}GetBuilder", output);
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
                    input: ::nitroglycerin::dynamodb::GetItemInput,
                    _phantom: ::std::marker::PhantomData<#phantom_data>,
                }

                impl #impl_generics #builder_p #ty_generics #where_clause {
                    fn new(client: #D, input: ::nitroglycerin::dynamodb::GetItemInput) -> Self {
                        Self { client, input, _phantom: ::std::marker::PhantomData }
                    }

                    #vis fn #s_ident(self, #s_ident: #s_ty) -> ::nitroglycerin::get::Expr<#D, #output #ty_generics2>
                    where
                        #s_ty: ::nitroglycerin::convert::IntoAttributeValue,
                    {
                        let sort_key = #s_ident;
                        let Self { client, mut input, _phantom } = self;

                        input.key.insert(
                            #s_name.to_owned(),
                            <#s_ty as ::nitroglycerin::convert::IntoAttributeValue>::into_av(sort_key),
                        );

                        ::nitroglycerin::get::Expr::new(client, input)
                    }
                }
            }),
            None => tokens.extend(quote! {
                #vis type #builder_p #impl_generics = ::nitroglycerin::get::Expr<#D, #output #ty_generics2>;
            }),
        }
    }
}
