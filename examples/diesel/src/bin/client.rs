use structopt::StructOpt;

use phalanx::web;

use diesel_example::{models::PostBuilder, BlogClient};

#[derive(StructOpt)]
enum Opts {
    Create {
        title: String,
        body: String,
    },
    Read {
        id: i32,
    },
    Update {
        id: i32,
        #[structopt(short, long)]
        title: Option<String>,
        #[structopt(short, long)]
        body: Option<String>,
        #[structopt(short, long)]
        published: bool,
    },
    Delete {
        id: i32,
    },
}

#[phalanx::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BlogClient::new("http://localhost:8080");

    let args = Opts::from_args();

    match args {
        Opts::Create { title, body } => {
            println!(
                "{:?}",
                client
                    .create_post(web::Json(PostBuilder {
                        title: Some(title),
                        body: Some(body),
                        published: Some(false)
                    }))
                    .await?
            );
        }
        Opts::Read { id } => {
            println!("{:?}", client.read_post(id).await?);
        }
        Opts::Update {
            id,
            title,
            body,
            published,
        } => {
            client
                .update_post(
                    id,
                    web::Json(PostBuilder {
                        title,
                        body,
                        published: Some(published),
                    }),
                )
                .await?
        }
        Opts::Delete { id } => client.delete_post(id).await?,
    }

    Ok(())
}
