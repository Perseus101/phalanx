use std::{future::Future, string::FromUtf8Error};

use err_derive::Error;

use futures::future::{err, ok, Ready};
use reqwest::{Client as ReqwestClient, Error as ReqwestError, Response};

use crate::util::AsyncTryFrom;

pub struct Client {
    pub client: ReqwestClient,
    pub url: String,
}

impl Client {
    pub fn new(client: ReqwestClient, url: String) -> Self {
        Client { client, url }
    }

    pub fn url(url: &str) -> Self {
        Self::new(ReqwestClient::default(), String::from(url))
    }

    pub fn format_url(&self, relative_url: &str) -> String {
        format!("{}{}", self.url, relative_url)
    }
}

impl From<String> for Client {
    fn from(url: String) -> Self {
        Self::new(ReqwestClient::default(), url)
    }
}

impl<'a> From<&'a str> for Client {
    fn from(url: &'a str) -> Self {
        Self::url(url)
    }
}

pub trait PhalanxClient {
    fn client(&self) -> &Client;
}

pub struct PhalanxResponse(pub Response);

impl From<Response> for PhalanxResponse {
    fn from(res: Response) -> Self {
        PhalanxResponse(res)
    }
}

#[derive(Debug, Error)]
pub enum PhalanxClientError {
    #[error(display = "error making request")]
    ReqwestError(#[error(source)] ReqwestError),
    #[error(display = "error parsing request")]
    ParseError(#[error(source)] FromUtf8Error),
    #[error(display = "error parsing request")]
    SerdeJsonError(#[error(source)] serde_json::Error),
}

type AsyncTryFromStringFuture = impl Future<Output = Result<String, PhalanxClientError>>;

impl AsyncTryFrom<PhalanxResponse> for String {
    type Error = PhalanxClientError;

    type Future = AsyncTryFromStringFuture;

    fn try_from(res: PhalanxResponse) -> Self::Future {
        async {
            let res = res.0.error_for_status()?;
            let bytes = res.bytes().await?;
            Ok(String::from_utf8(Vec::from(&bytes[..]))?)
        }
    }
}

impl AsyncTryFrom<PhalanxResponse> for () {
    type Error = PhalanxClientError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn try_from(res: PhalanxResponse) -> Self::Future {
        match res.0.error_for_status() {
            Err(e) => err(e.into()),
            Ok(_) => ok(()),
        }
    }
}

#[allow(non_camel_case_types)]
pub enum ContentType {
    TEXT_PLAIN,
    APPLICATION_JSON,
}

impl ContentType {
    pub fn header_value(self) -> &'static str {
        match self {
            ContentType::TEXT_PLAIN => "text/plain",
            ContentType::APPLICATION_JSON => "application/json",
        }
    }
}

impl From<&String> for ContentType {
    fn from(_: &String) -> Self {
        Self::TEXT_PLAIN
    }
}
