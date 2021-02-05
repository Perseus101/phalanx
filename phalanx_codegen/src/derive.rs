use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Error, Index};

pub fn derive_serialize_inner(input: DeriveInput) -> Result<TokenStream, Error> {
    let client_type = input.ident;
    match input.data {
        Data::Struct(s) => {
            let mut client_index: Option<usize> = None;
            let mut client_ident: Option<Ident> = None;
            for (i, field) in s.fields.iter().enumerate() {
                for attr in field.attrs.iter() {
                    if attr.path.is_ident("client") {
                        client_index = Some(i);
                        client_ident = field.ident.clone();
                        break;
                    }
                }
            }

            match &s.fields {
                syn::Fields::Named(_named) => {
                    let client_ident = client_ident.ok_or_else(|| {
                        Error::new(
                            Span::call_site(),
                            "Missing a `#[client]` attribute on the client field",
                        )
                    })?;

                    let output = quote! {
                        impl phalanx::client::PhalanxClient for #client_type {
                            fn client(&self) -> &phalanx::client::Client {
                                &self. #client_ident
                            }
                        }
                    };
                    Ok(output.into())
                }
                syn::Fields::Unnamed(_unnamed) => {
                    let client_index = Index::from(client_index.ok_or_else(|| {
                        Error::new(
                            Span::call_site(),
                            "Missing a `#[client]` attribute on the client field",
                        )
                    })?);

                    let output = quote! {
                        impl phalanx::client::PhalanxClient for #client_type {
                            fn client(&self) -> &phalanx::client::Client {
                                &self. #client_index
                            }
                        }
                    };
                    Ok(output.into())
                }
                syn::Fields::Unit => Err(Error::new(
                    Span::call_site(),
                    "PhalanxClient cannot be derived on unit structs",
                )),
            }
        }
        Data::Enum(_) | Data::Union(_) => Err(Error::new(
            Span::call_site(),
            "PhalanxClient can only be derived on structs",
        )),
    }
}
