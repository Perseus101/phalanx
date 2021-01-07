use phalanx::client::{Client, PhalanxClient};
use phalanx::prelude::*;

#[derive(Clone)]
pub struct SimpleServer {}

pub struct SimpleClient {
    client: Client,
}

impl PhalanxClient for SimpleClient {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl SimpleClient {
    pub fn new(url: &str) -> Self {
        SimpleClient {
            client: Client::from(url),
        }
    }
}

#[phalanx_client(SimpleClient)]
#[phalanx_server]
impl SimpleServer {
    #[get("/{id}/{name}/index.html")]
    async fn index(&self, id: u32, name: String) -> String {
        format!("Hello {}! id:{}", name, id)
    }
}

// impl SimpleClient {
//     pub async fn index(&self, id: u32, name: String) -> Result<String, Box<dyn std::error::Error>> {
//         let client = phalanx::client::PhalanxClient::client(self);
//         Ok(String::try_from(PhalanxResponse::from(
//             client
//                 .client
//                 .get(&client.format_url(&format!("/{id}/{name}/index.html", id = id, name = name)))
//                 .send()
//                 .await?,
//         ))
//         .await?)
//     }
// }

// #[get("/{id}/{name}/index.html")]
// async fn index(
//     data: web::Data<SimpleServer>,
//     web::Path((id, name)): web::Path<(u32, String)>,
// ) -> impl Responder {
//     data.into_inner().index(id, name)
// }

// impl PhalanxServer for SimpleServer {
//     fn mount(config: &mut web::ServiceConfig) {
//         config.service(index);
//     }
// }
