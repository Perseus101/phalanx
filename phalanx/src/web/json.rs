use std::{convert::TryFrom, fmt, ops};

use actix_web::{web::JsonConfig, FromRequest, HttpRequest, HttpResponse, Responder};
use futures::{future::Ready, FutureExt};
use reqwest::Body;
use serde::Serialize;

/// Struct wrapping actix_web's [Json](actix_web::web::Json) struct
pub struct Json<T>(pub T);

impl<T: Serialize> TryFrom<Json<T>> for Body {
    type Error = serde_json::Error;

    fn try_from(value: Json<T>) -> Result<Self, Self::Error> {
        let vec = serde_json::to_vec(&value.0)?;
        Ok(Body::from(vec))
    }
}

impl<T> Json<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Json<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Json: {:?}", self.0)
    }
}

impl<T> fmt::Display for Json<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<T: Serialize> Responder for Json<T> {
    type Error = actix_web::Error;
    type Future = Ready<Result<HttpResponse, actix_web::Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        actix_web::web::Json(self.0).respond_to(req)
    }
}

type JsonFromRequestFuture<T> =
    impl std::future::Future<Output = Result<Json<T>, actix_web::Error>>;
/// Json extractor. Allow to extract typed information from request's payload.
impl<T> FromRequest for Json<T>
where
    T: serde::de::DeserializeOwned + 'static,
{
    type Error = actix_web::Error;
    type Future = JsonFromRequestFuture<T>;
    type Config = JsonConfig;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        actix_web::web::Json::<T>::from_request(req, payload)
            .map(|res| res.map(|json| Json(json.into_inner())))
    }
}
