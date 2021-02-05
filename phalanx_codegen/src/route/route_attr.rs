use std::convert::TryFrom;

use syn::spanned::Spanned;

use super::*;

macro_rules! method_type {
        (
            $($variant:ident, $lower:ident,)+
        ) => {
            #[derive(Debug, PartialEq, Eq, Hash, Clone)]
            pub(super) enum MethodType {
                $(
                    $variant,
                )+
            }

            impl MethodType {
                fn as_str(&self) -> &'static str {
                    match self {
                        $(Self::$variant => stringify!($variant),)+
                    }
                }

                fn as_lower_str(&self) -> &'static str {
                    match self {
                        $(Self::$variant => stringify!($lower),)+
                    }
                }


                fn parse(method: &str) -> Result<Self, String> {
                    match method {
                        $(stringify!($lower) => Ok(Self::$variant),)+
                        _ => Err(format!("Unexpected HTTP method: `{}`", method)),
                    }
                }
            }
        };
    }

method_type! {
    Get,       get,
    Post,      post,
    Put,       put,
    Delete,    delete,
    Head,      head,
    Connect,   connect,
    Options,   options,
    Trace,     trace,
    Patch,     patch,
}

#[derive(Clone)]
pub(super) struct RouteAttr {
    pub method: MethodType,
    pub route: LitStr,
    span: Span,
}

impl RouteAttr {
    pub fn method_ident(&self) -> Ident {
        Ident::new(self.method.as_str(), self.span)
    }

    pub fn method_ident_lower(&self) -> Ident {
        Ident::new(self.method.as_lower_str(), self.span)
    }
}

impl TryFrom<&Attribute> for RouteAttr {
    type Error = syn::Error;

    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        struct RawRoute {
            route: LitStr,
        }

        impl Parse for RawRoute {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                Ok(Self {
                    route: input.parse()?,
                })
            }
        }

        let method_string = &attr.path.segments[attr.path.segments.len() - 1]
            .ident
            .to_string();

        let method =
            MethodType::parse(method_string).map_err(|err| syn::Error::new_spanned(attr, err))?;
        let RawRoute { route } = attr.parse_args::<RawRoute>()?;
        Ok(Self {
            method,
            route,
            span: attr.span(),
        })
    }
}
