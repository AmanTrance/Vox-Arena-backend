mod routes;
mod models;
mod schema;
mod utils;
mod socket;
use crate::schema::users::dsl::users;
use actix_web::{dev::{Service, Url}, http::{Method, Uri}, web::{self}, App, HttpServer};
use diesel::{self, RunQueryDsl, query_dsl::methods::FilterDsl, Connection, ExpressionMethods, PgConnection};
use models::models::User;
use routes::routes::{hello, initialize_ws, unauthorized, user_signin, user_signup};
use dotenv::dotenv;
use schema::users::token;
use socket::manager::ArenaHandler;
use tokio::spawn;
use std::{env, sync::{Arc, Mutex}};

pub struct DBState{
    client: Arc<Mutex<PgConnection>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let databse_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = Arc::new(Mutex::new(PgConnection::establish(&databse_url).expect("Database not connected!")));
    let arena_handler = ArenaHandler::new();
    spawn(arena_handler.0.run());

    HttpServer::new(move || {
        let connection_clone = connection.clone();
        App::new()
            .app_data(web::Data::new(DBState{
                client: Arc::clone(&connection)
            }))
            .app_data(web::Data::new(arena_handler.1.clone()))
            .wrap_fn(move |mut req, srv| {
                if req.path() != "/api/signup" && req.path() != "/api/signin" {
                    let headers = req.headers();
                    match headers.get("Authorization") {
                        Some(auth_token) => {
                            let user: Vec<User> = users.filter(token.eq(auth_token.to_str().unwrap())).load(&mut *connection_clone.lock().unwrap()).expect("Databse crashed");
                            if user.len() != 0 {
                                srv.call(req)
                            } else {
                                req.head_mut().method = Method::GET;
                                req.match_info_mut().set(Url::new(Uri::from_static("/api/unauthorized")));
                                srv.call(req)
                            }
                            
                        },
                        None => {
                            req.head_mut().method = Method::GET;
                            req.match_info_mut().set(Url::new(Uri::from_static("/api/unauthorized")));
                            srv.call(req)
                        }
                    }
                } else {
                    srv.call(req)
                }
            })
            .service(web::scope("/api")
            .service(hello)
            .service(user_signup)
            .service(user_signin)
            .service(unauthorized)
            .service(initialize_ws)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}