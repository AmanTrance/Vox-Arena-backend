mod routes;
mod models;
mod schema;
mod utils;
extern crate uuid;
extern crate actix_web;
extern crate dotenv;
extern crate diesel;
use actix_web::{web::{self}, App, HttpServer};
use diesel::{Connection, PgConnection};
use routes::routes::{hello, user_signup, user_signin};
use dotenv::dotenv;
use std::{env, sync::{Arc, Mutex}};


pub struct DBState{
    client: Arc<Mutex<PgConnection>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let databse_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = Arc::new(Mutex::new(PgConnection::establish(&databse_url).expect("Database not connected!")));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(DBState{
                client: Arc::clone(&connection)
            }))
            .service(web::scope("/api")
            .service(hello)
            .service(user_signup)
            .service(user_signin)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}