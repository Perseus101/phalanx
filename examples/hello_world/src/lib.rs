use phalanx::client::Client;
use phalanx::prelude::*;

#[derive(Clone)]
pub struct SimpleServer {}

#[derive(PhalanxClient)]
pub struct SimpleClient {
    #[client]
    client: Client,
}

impl SimpleClient {
    pub fn new(url: &str) -> Self {
        SimpleClient {
            client: Client::from(url),
        }
    }
}

#[phalanx(SimpleClient)]
impl SimpleServer {
    #[get("/{name}/index.html")]
    async fn index(&self, name: String) -> String {
        format!("Hello {}!", name)
    }

    #[post("/foo")]
    async fn foo(&self, name: String) -> String {
        format!("Hello {}!", name)
    }
}

// impl SimpleClient {
//     pub async fn _index(&self, name: String) -> Result<String, Box<dyn std::error::Error>> {
//         use phalanx::util::AsyncTryFrom;
//         let client = phalanx::client::PhalanxClient::client(self);
//         Ok(String::try_from(phalanx::client::PhalanxResponse::from(
//             client
//                 .client
//                 .get(&client.format_url(&format!("/{name}/index.html", name = name)))
//                 .send()
//                 .await?,
//         ))
//         .await?)
//     }

//     pub async fn _foo(&self, name: String) -> Result<String, Box<dyn std::error::Error>> {
//         use phalanx::util::AsyncTryFrom;
//         let client = phalanx::client::PhalanxClient::client(self);
//         let body: reqwest::Body = std::convert::TryFrom::try_from(name)?;
//         Ok(String::try_from(phalanx::client::PhalanxResponse::from(
//             client
//                 .client
//                 .post(&client.format_url(&format!("/foo")))
//                 .body(body)
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
