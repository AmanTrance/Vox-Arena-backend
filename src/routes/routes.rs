use crate::{models::models::{Any, NewUser, ReturnUser, SolBalance, User}, schema::users::{email, token, username}, socket::manager::Command, DBState};
use crate::utils::response::{ErrorResponse, ApiResponse};
use crate::socket::ws::handle_ws;
use actix_web::{get, post, put, web::{self, Data, Json}, HttpRequest, HttpResponse, Responder};
use diesel::{self, query_dsl::methods::FilterDsl, ExpressionMethods, RunQueryDsl};
use tokio::{sync::mpsc, task::spawn_local};
use uuid::Uuid;
use bcrypt::{hash, verify};
use actix_ws::handle;


#[get("/hello")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World!")
}

#[post("/signup")]
pub async fn user_signup(db: web::Data<DBState>, mut user_data: Json<NewUser>) -> impl Responder {
    use crate::schema::users::dsl::users;
    match web::block(move || {
    let user1: Vec<User> = users.filter(username.eq(&user_data.username)).load(&mut *db.client.lock().unwrap()).expect("Database crashed");
    let user2: Vec<User> = users.filter(email.eq(&user_data.email)).load(&mut *db.client.lock().unwrap()).expect("Database crashed");
    user_data.password = hash(&user_data.password, 14).unwrap();
        ((user1, user2), db, user_data)
     }).await {
        Ok(user) => {
            if user.0.0.len() > 0 || user.0.1.len() > 0 {
                HttpResponse::Conflict().json(ApiResponse {
                    data: "username or email already exists"
                })
            } else {
                match web::block(move || {
                    let mut connection = user.1.client.lock().unwrap();
                    let user_id: i32 = diesel::insert_into(crate::schema::users::table)
                    .values(user.2.into_inner())
                    .returning(crate::schema::users::id)
                    .get_result(&mut *connection)
                    .expect("User not created");
                    diesel::insert_into(crate::schema::sol_balance::table)
                    .values(SolBalance{ balance: 0.0, user_id})
                    .execute(&mut *connection)
                    .expect("Database crashed");
                }).await {
                    Ok(_) => HttpResponse::Created().json(ApiResponse {
                        data: "user created"
                    }),
                    Err(e) => HttpResponse::InternalServerError().json(ErrorResponse{
                        error: &e.to_string()
                    })
                }
            }
            
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse{
            error: &e.to_string()
        })
    }
}

#[post("/signin")]
pub async fn user_signin(db: Data<DBState>, credentials: Json<Any>) -> impl Responder {
    use crate::schema::users::dsl::users;
    let mut connection = db.client.lock().unwrap();
        match credentials.into_inner() {
            Any::EmailWithPassword(x, y) => {
                let user_by_email: Vec<User> = users.filter(email.eq(&x)).load(&mut *connection).expect("Database crashed");
                if user_by_email.len() == 0 {
                    HttpResponse::BadRequest().json(ApiResponse {
                        data: "user does not exist"
                    })
                } else {
                    if verify(&y, &user_by_email[0].password).unwrap() {
                        let auth_token: String = Uuid::new_v4().to_string();
                        diesel::update(crate::schema::users::table)
                                .filter(email.eq(&user_by_email[0].email))
                                .set(token.eq(&auth_token))
                                .execute(&mut *connection)
                                .expect("Database crashed");
                        HttpResponse::Ok().json(ApiResponse {
                            data: ReturnUser {
                                username: &user_by_email[0].username,
                                token: &auth_token
                            }
                        })
                    } else {
                        HttpResponse::BadRequest().json(ApiResponse {
                            data: "password is incorrect"
                        })
                    }
                }
            },
            Any::UsernameWithPassword(x, y) => {
                let user_by_username: Vec<User> = users.filter(username.eq(&x)).load(&mut *connection).expect("Database crashed");
                if user_by_username.len() == 0 {
                    HttpResponse::BadRequest().json(ApiResponse {
                        data: "user does not exist"
                    })
                } else {
                    if verify(&y, &user_by_username[0].password).unwrap() {
                        let auth_token: String = Uuid::new_v4().to_string();
                        diesel::update(crate::schema::users::table)
                        .filter(email.eq(&user_by_username[0].email))
                        .set(token.eq(&auth_token))
                        .execute(&mut *connection)
                        .expect("Database crashed");
                        HttpResponse::Ok().json(ApiResponse {
                            data: ReturnUser {
                                username: &user_by_username[0].username,
                                token: &auth_token
                            }
                        })
                    } else {
                        HttpResponse::BadRequest().json(ApiResponse {
                            data: "password is incorrect"
                        })
                    }
                }
            }
        }
}

#[get("/unauthorized")]
pub async fn unauthorized() -> impl Responder {
    HttpResponse::BadRequest().json(ApiResponse {
        data: "unauthorized"
    })
}

#[get("/ws")]
pub async fn initialize_ws(sender: web::Data<mpsc::Sender<Command>>, req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, message) = handle(&req, stream)?;
    spawn_local(handle_ws(sender.into_inner(), session, message));
    Ok(response)
}

// #[put("/payment")]
pub async fn _solana_balance_update() -> () {
    todo!()
}