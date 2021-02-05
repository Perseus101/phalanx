use proc_macro::TokenStream;
use quote::{quote, ToTokens};

use syn::{parse_macro_input, Error, ImplItem, ItemImpl, Type};

use crate::route::{ClientRoute, Route, ServerRoute};

/// Wrapper for a single service
pub struct Service {
    server: ServerService,
    client: ClientService,
    parsed_impl: ItemImpl,
}

impl Service {
    pub fn from_tokens(attr: TokenStream, input: TokenStream) -> TokenStream {
        let parsed_impl = parse_macro_input!(input as ItemImpl);
        let service = match Self::new(attr, parsed_impl) {
            Ok(s) => s,
            Err(e) => return e.to_compile_error().into(),
        };

        let output = quote! {
            #service
        };

        output.into()
    }

    fn new(attr: TokenStream, parsed_impl: ItemImpl) -> Result<Self, Error> {
        validate_impl(&parsed_impl)?;
        let routes = parse_routes(&parsed_impl)?;

        let client_routes: Vec<ClientRoute> = routes
            .iter()
            .map(|route| ClientRoute::from(route.clone()))
            .collect();

        let server_routes: Vec<ServerRoute> = routes
            .into_iter()
            .map(|route| ServerRoute::from(route))
            .collect();

        let server_type = parsed_impl.self_ty.as_ref();

        Ok(Service {
            server: ServerService::new(server_routes, server_type.clone()),
            client: ClientService::from_attr(attr, client_routes)?,
            parsed_impl,
        })
    }
}

struct ClientService {
    ty: Type,
    routes: Vec<ClientRoute>,
}

impl ClientService {
    fn from_attr(attr: TokenStream, client_routes: Vec<ClientRoute>) -> Result<Self, Error> {
        let client_type: Type = syn::parse(attr).map_err(|err| {
            syn::Error::new(
                err.span(),
                "phalanx requires a client type, i.e. `#[phalanx(MyClient)]`",
            )
        })?;
        Ok(Self {
            ty: client_type,
            routes: client_routes,
        })
    }
}

struct ServerService {
    routes: Vec<ServerRoute>,
    server_type: Type,
}

impl ServerService {
    fn new(server_routes: Vec<ServerRoute>, server_type: Type) -> Self {
        Self {
            routes: server_routes,
            server_type,
        }
    }
}

impl ToTokens for Service {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let parsed_impl = &self.parsed_impl;
        let server = &self.server;
        let client = &self.client;

        tokens.extend(quote! {
            #parsed_impl

            #server

            #client
        });
    }
}

impl ToTokens for ClientService {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let routes = &self.routes;
        let client_type = &self.ty;

        tokens.extend(quote! {
            impl #client_type {
                #(#routes)*
            }
        });
    }
}

impl ToTokens for ServerService {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let routes = &self.routes;
        let server_type = &self.server_type;

        tokens.extend(quote! {
            impl phalanx::server::PhalanxServer for #server_type {
                fn mount(__config: &mut phalanx::reexports::web::ServiceConfig) {
                    #(#routes)*
                }
            }
        });
    }
}

fn validate_impl(parsed_impl: &ItemImpl) -> syn::Result<()> {
    if parsed_impl.defaultness.is_some() {
        return Err(syn::Error::new_spanned(
            &parsed_impl.defaultness,
            "Default impls are not currently supported by phalanx_server.",
        ));
    }

    if !parsed_impl.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &parsed_impl.generics,
            "Generics are not supported by phalanx_server.",
        ));
    }

    Ok(())
}

fn parse_routes(parsed_impl: &ItemImpl) -> syn::Result<Vec<Route>> {
    let mut routes: Vec<Route> = Vec::new();
    let server_type = parsed_impl.self_ty.as_ref();

    for item in &parsed_impl.items {
        match item {
            ImplItem::Method(method) => match Route::new(method, server_type) {
                Ok(route) => routes.push(route),
                Err(err) => return Err(err),
            },
            ImplItem::Type(assoc_type) => {
                return Err(syn::Error::new_spanned(
                    &assoc_type,
                    "Associated Types are not currently supported by phalanx_server.",
                ));
            }
            _ => (),
        }
    }

    Ok(routes)
}
