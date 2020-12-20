use super::reexports;

pub trait PhalanxServer: Clone {
    fn mount(config: &mut reexports::web::ServiceConfig);
}
