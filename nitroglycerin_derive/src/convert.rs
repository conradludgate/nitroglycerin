use quote::{ToTokens, quote};
use syn::Ident;

use crate::query::Column;


pub struct ConvertBuilder {
    pub ident: Ident,
    pub columns: Vec<Column>,
}

impl ToTokens for ConvertBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { ident, columns } = self;

        let idents = columns.iter().map(|c| &c.ident);
        let names = columns.iter().map(|c| &c.name);
        let tys = columns.iter().map(|c| &c.ty);

        tokens.extend(quote! {
            impl ::std::convert::From<#ident> for ::nitroglycerin::Attributes {
                fn from(t: #ident) -> Self {
                    <_>::into_iter([
                        #(
                            (#names.to_owned(), <#tys as ::nitroglycerin::convert::IntoAttributeValue>::into_av(t.#idents)),
                        )*
                    ]).collect()
                }
            }
        });

        let idents = columns.iter().map(|c| &c.ident);
        let names = columns.iter().map(|c| &c.name);
        let tys = columns.iter().map(|c| &c.ty);

        tokens.extend(quote!{
            impl ::std::convert::TryFrom<::nitroglycerin::Attributes> for #ident {
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
