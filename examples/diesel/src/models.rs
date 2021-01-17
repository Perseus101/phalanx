use diesel::Queryable;
use serde::{Deserialize, Serialize};

use super::schema::posts;

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "posts"]
pub struct PostBuilder {
    pub title: Option<String>,
    pub body: Option<String>,
    pub published: Option<bool>,
}
