use quote::{quote, ToTokens};
use syn::{parse_quote, Generics, Ident};

use crate::table::Column;

pub struct ConvertBuilder {
    from: FromBuilder,
    into: IntoBuilder,
}

impl ConvertBuilder {
    pub fn new(ident: Ident, generics: Generics, columns: Vec<Column>) -> Self {
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
        });
    }
}
