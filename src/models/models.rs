use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn uuid() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, PartialEq, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub token: String
}

#[derive(Debug, Insertable, Deserialize, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default="uuid")]
    pub token: String
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Any {
    UsernameWithPassword(String, String),
    EmailWithPassword(String, String)
}

#[derive(Debug, PartialEq, Serialize)]
pub struct ReturnUser<'a> {
    pub username: &'a str,
    pub token: &'a str
}

#[derive(Debug, PartialEq, Queryable, Selectable, Serialize, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::sol_balance)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SolBalance {
    balance: f64,
    user_id: i32
}

