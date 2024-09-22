use crate::{models::models::{Any, NewUser, ReturnUser, User}, schema::users::{email, token, username}, DBState};
use crate::utils::response::{ErrorResponse, ApiResponse};
use actix_web::{get, post, web::{self, Data, Json}, HttpResponse, Responder};
use diesel::{self, query_dsl::methods::FilterDsl, ExpressionMethods, RunQueryDsl};
use uuid::Uuid;
use bcrypt::{hash, verify};


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
                    diesel::insert_into(crate::schema::users::table)
                    .values(user.2.into_inner())
                    .execute(&mut *connection)
                    .expect("User not created");
                    drop(connection);
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