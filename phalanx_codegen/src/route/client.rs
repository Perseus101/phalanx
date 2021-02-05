use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

use super::Route;
pub struct ClientRoute(Route);

impl From<Route> for ClientRoute {
    fn from(route: Route) -> Self {
        ClientRoute(route)
    }
}

impl ToTokens for ClientRoute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let args = &self.0.args;
        let fn_name = &self.0.ident;
        let raw_ret_type = &self.0.ret_type;
        let attrs = &self.0.attrs;
        let route = &self.0.route_attr.route;
        let method = self.0.route_attr.method_ident_lower();

        // Get just the arguments which affect the path of the request
        let path_args = super::split_args(&self.0.path_args);

        let format_args: Vec<TokenStream2> = path_args
            .iter()
            .map(|(ident, _)| quote! { #ident = #ident })
            .collect();

        let format_url = if format_args.len() > 0 {
            quote! { &format!( #route , #(#format_args),* ) }
        } else {
            quote! { #route }
        };

        // Ensure the type is a result type
        let ret_type = match raw_ret_type {
            syn::ReturnType::Default => {
                quote! { () }
            }
            syn::ReturnType::Type(_, ty) => {
                quote! { #ty }
            }
        };

        let (content_type, payload) = if let Some(payload) = &self.0.payload_arg {
            match payload.pat.as_ref() {
                syn::Pat::Ident(ident) => (
                    quote! {
                        let __content_type = phalanx::client::ContentType::from(&#ident);
                    },
                    quote! {
                        .header("content-type", __content_type.header_value())
                        .body({
                            let body: phalanx::reexports::Body = std::convert::TryFrom::try_from(#ident)?;
                            body
                        })
                    },
                ),
                pat => panic!("Unknown pattern: {:?}", pat),
            }
        } else {
            (quote! {}, quote! {})
        };

        let stream = quote! {
            #(#attrs)*
            pub async fn #fn_name ( &self, #(#args),* ) -> Result< #ret_type , Box<dyn std::error::Error> > {
                let __client  = phalanx::client::PhalanxClient::client(self);
                #content_type
                let __req = __client.client. #method (&__client.format_url( #format_url )) #payload;
                let __res = phalanx::client::PhalanxResponse::from(__req.send().await?);
                Ok(<#ret_type as phalanx::util::AsyncTryFrom<phalanx::client::PhalanxResponse>>::try_from(__res).await?)
            }
        };

        tokens.extend(stream);
    }
}
