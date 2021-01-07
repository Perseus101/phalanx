use phalanx::client::PhalanxClient;
use phalanx::prelude::get;

use phalanx_codegen::{phalanx_client, phalanx_server};

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

    #[phalanx_client(NoArgClient)]
    #[phalanx_server]
    impl NoArgServer {
        #[get("/")]
        async fn index(&self) {}
    }

    fn _test() {
        let client = NoArgClient(Client::url("http://localhost:8080"));
        let _future = client.index();
    }
}
