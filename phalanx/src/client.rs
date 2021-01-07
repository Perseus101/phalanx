use std::string::FromUtf8Error;

use async_trait::async_trait;
use err_derive::Error;

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

pub struct PhalanxResponse(Response);

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
    ParseError(Box<dyn std::error::Error>),
}

impl From<FromUtf8Error> for PhalanxClientError {
    fn from(err: FromUtf8Error) -> Self {
        PhalanxClientError::ParseError(err.into())
    }
}
#[async_trait]
impl AsyncTryFrom<PhalanxResponse> for String {
    type Error = PhalanxClientError;

    async fn try_from(res: PhalanxResponse) -> Result<Self, Self::Error> {
        Ok(String::from_utf8(Vec::from(&res.0.bytes().await?[..]))?)
    }
}

#[async_trait]
impl AsyncTryFrom<PhalanxResponse> for () {
    type Error = PhalanxClientError;

    async fn try_from(res: PhalanxResponse) -> Result<Self, Self::Error> {
        res.0.error_for_status()?;
        Ok(())
    }
}
