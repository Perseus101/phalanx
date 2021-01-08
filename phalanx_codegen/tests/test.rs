use phalanx_codegen::{get, phalanx, PhalanxClient};

mod noargs {
    use super::*;
    use phalanx::client::Client;

    #[derive(Clone)]
    struct NoArgServer;

    #[derive(PhalanxClient)]
    struct NoArgClient(#[client] Client);

    #[phalanx(NoArgClient)]
    impl NoArgServer {
        #[get("/")]
        async fn index(&self) {}
    }

    fn _test() {
        let client = NoArgClient(Client::url("http://localhost:8080"));
        let _future = client.index();
    }
}
