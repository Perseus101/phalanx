#![feature(type_alias_impl_trait)]

pub mod client;
pub mod server;
pub mod util;
pub mod web;

pub use phalanx_codegen::{main, phalanx, PhalanxClient};

pub mod prelude {
    pub use crate::server::{mount::PhalanxMount, PhalanxServer};

    pub use phalanx_codegen::{connect, delete, get, head, options, patch, post, put, trace};
    pub use phalanx_codegen::{phalanx, PhalanxClient};
}

pub mod reexports {
    pub use actix_web::{
        guard, http, middleware, rt, web, App, HttpResponse, HttpServer, Resource, Responder,
    };

    pub use reqwest::{Body, Client};
}
