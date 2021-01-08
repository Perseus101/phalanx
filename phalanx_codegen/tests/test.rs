use phalanx::client::PhalanxClient;

use phalanx_codegen::{get, phalanx};

mod noargs {
    use super::*;
    use phalanx::client::Client;

    #[derive(Clone)]
    struct NoArgServer;

    struct NoArgClient(Client);

    impl PhalanxClient for NoArgClient {
        fn client(&self) -> &Client {
            &self.0
        }
    }

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
