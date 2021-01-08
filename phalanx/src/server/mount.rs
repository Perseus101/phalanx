use actix_service::ServiceFactory;
use actix_web::{
    dev::{MessageBody, ServiceRequest, ServiceResponse},
    App,
};

use super::PhalanxServer;

/// This trait adds a configuration method to [App](actix_web::App)
/// specifically for configuring Phalanx services
pub trait PhalanxMount: Sized {
    fn phalanx_mount<S: PhalanxServer + 'static>(self, service: S) -> Self;
}

impl<T, B> PhalanxMount for App<T, B>
where
    B: MessageBody,
    T: ServiceFactory<
        Config = (),
        Request = ServiceRequest,
        Response = ServiceResponse<B>,
        Error = actix_web::Error,
        InitError = (),
    >,
{
    fn phalanx_mount<S: PhalanxServer + 'static>(self, service: S) -> Self {
        self.configure(S::mount).data(service)
    }
}
