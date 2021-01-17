#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use phalanx::prelude::*;
use phalanx::{client::Client, web};

pub mod models;
pub mod schema;

use models::{Post, PostBuilder};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct BlogServer {
    pool: DbPool,
}

impl BlogServer {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[derive(PhalanxClient)]
pub struct BlogClient {
    #[client]
    client: Client,
}

impl BlogClient {
    pub fn new(url: &str) -> Self {
        Self {
            client: Client::from(url),
        }
    }
}

no_arg_sql_function!(last_insert_rowid, diesel::sql_types::Integer);

#[phalanx(BlogClient)]
impl BlogServer {
    #[post("/post")]
    async fn create_post(&self, new_post: web::Json<PostBuilder>) -> web::Json<Post> {
        use crate::schema::posts::{self, dsl::*};

        let conn = self.pool.get().expect("conn");
        let new_post = new_post.into_inner();

        diesel::insert_into(posts::table)
            .values(&new_post)
            .execute(&conn)
            .expect("Error saving new post");

        let post_id: i32 = diesel::select(last_insert_rowid).first(&conn).unwrap();

        let post = posts
            .filter(id.eq(post_id))
            .first(&conn)
            .expect("Error loading posts");

        web::Json(post)
    }

    #[get("/post/{post_id}")]
    async fn read_post(&self, post_id: i32) -> web::Json<Post> {
        use crate::schema::posts::dsl::*;

        let conn = self.pool.get().expect("conn");

        let post = posts
            .filter(id.eq(post_id))
            .first(&conn)
            .expect("Error loading posts");

        web::Json(post)
    }

    #[put("/post/{post_id}")]
    async fn update_post(&self, post_id: i32, post: web::Json<PostBuilder>) {
        use crate::schema::posts::dsl::*;

        let post = post.into_inner();
        let (title_, body_, published_) = match (post.title, post.body, post.published) {
            (None, None, None) => return,
            (title_, body_, published_) => (title_, body_, published_),
        };

        let conn = self.pool.get().expect("conn");

        let stmt = diesel::update(posts.find(post_id));

        let stmt = match (title_, body_, published_) {
            (None, None, None) => return,
            (Some(title_), None, None) => stmt.set(title.eq(title_)).execute(&conn),
            (None, Some(body_), None) => stmt.set(body.eq(body_)).execute(&conn),
            (None, None, Some(published_)) => stmt.set(published.eq(published_)).execute(&conn),

            (Some(title_), Some(body_), None) => {
                stmt.set((title.eq(title_), body.eq(body_))).execute(&conn)
            }
            (None, Some(body_), Some(published_)) => stmt
                .set((body.eq(body_), published.eq(published_)))
                .execute(&conn),
            (Some(title_), None, Some(published_)) => stmt
                .set((title.eq(title_), published.eq(published_)))
                .execute(&conn),
            (Some(title_), Some(body_), Some(published_)) => stmt
                .set((title.eq(title_), body.eq(body_), published.eq(published_)))
                .execute(&conn),
        };

        stmt.expect(&format!("Unable to find post {}", post_id));
    }

    #[delete("/post/{post_id}")]
    async fn delete_post(&self, post_id: i32) {
        use crate::schema::posts::dsl::*;

        let conn = self.pool.get().expect("conn");

        diesel::delete(posts.find(post_id))
            .execute(&conn)
            .expect(&format!("Unable to find post {}", post_id));
    }
}

pub fn establish_connection() -> SqliteConnection {
    SqliteConnection::establish("test.db").expect("Error connecting to database")
}
