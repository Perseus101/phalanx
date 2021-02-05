use actix_web::{dev::Body, error::Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ok, Ready};

pub mod mount;

pub trait PhalanxServer: Clone {
    fn mount(config: &mut actix_web::web::ServiceConfig);
}

/// A special responder for the unit type
/// Used for convenience in phalanx_codegen
pub struct UnitResponder;

impl Responder for UnitResponder {
    type Error = Error;

    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        ok(HttpResponse::build(actix_web::http::StatusCode::OK).body(Body::Empty))
    }
}
