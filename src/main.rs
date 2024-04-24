use diesel::result::Error::NotFound;
use std::error::Error;

use rocket::http::Method;
use rocket::{get, routes};
use rocket::{
    response::status::Custom,
    serde::json::{json, Json, Value},
};
use rocket_cors::{AllowedHeaders, AllowedMethods, AllowedOrigins};
mod auth;
mod controllers;
mod models;
mod schema;
use crate::models::Link;
use crate::models::NewLink;
use crate::models::NewUser;
use crate::models::User;
use auth::BasicAuth;
use controllers::LinkController;
use controllers::UserController;
use controllers::Verifier;
use rocket::http::Status;
use rocket_sync_db_pools::database;
#[macro_use]
extern crate rocket;

#[database("sqlite")]

struct DbConn(diesel::SqliteConnection);

// pub trait SqliteConnectionExt {
//     fn find_user(&mut self, email: &str, password: &str) -> QueryResult<User>;
// }

// impl SqliteConnectionExt for SqliteConnection {
//     fn find_user(&mut self, email: &str, password: &str) -> QueryResult<User> {
//         users::table
//             .filter(users::email.eq(email))
//             .filter(users::password.eq(password))
//             .first(self)
//     }
// }
//
// #[get("/test")]
// async fn find_user_route(auth: BasicAuth) -> Result<Value, Custom<Value>> {
//     let database_url = "sqlite://./database.sqlite";
//     let mut connection =
//         SqliteConnection::establish(&database_url).expect("Failed to establish connection");

//     match connection.find_user(&auth.email, &auth.password) {
//         Ok(_user) => match UserController::find_many(&mut connection) {
//             Ok(users) => Ok(json!(users)),
//             Err(e) => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
//         },
//         Err(e) => match e {
//             NotFound => Err(Custom(Status::NotFound, json!(e.to_string()))),
//             _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
//         },
//     }
// }

//USERS

#[get("/users")]
async fn all_users(db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(|c| match UserController::find_many(c) {
        Ok(users) => Ok(json!(users)),
        Err(e) => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
    })
    .await
}

#[get("/users/<id>")]
async fn user(db: DbConn, id: i32) -> Result<Value, Custom<Value>> {
    db.run(move |c| match UserController::find_one(c, id) {
        Ok(users) => Ok(json!(users)),
        Err(e) => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
    })
    .await
}

#[post("/users", format = "json", data = "<new_user>")]
async fn create_user(db: DbConn, new_user: Json<NewUser>) -> Result<Value, Custom<Value>> {
    db.run(|c| match UserController::create(c, new_user.into_inner()) {
        Ok(user) => Ok(json!(user)),
        Err(e) => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
    })
    .await
}

#[put("/users/<id>", format = "json", data = "<user>")]
async fn update_user(db: DbConn, user: Json<User>, id: i32) -> Result<Value, Custom<Value>> {
    db.run(
        move |c| match UserController::update(c, user.into_inner(), id) {
            Ok(users) => Ok(json!(users)),
            Err(e) => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
        },
    )
    .await
}

#[delete("/users/<id>")]
async fn delete_user(db: DbConn, id: i32) -> Result<Value, Custom<Value>> {
    db.run(move |c| match UserController::delete(c, id) {
        Ok(users) => Ok(json!(users)),
        Err(e) => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
    })
    .await
}

//LINKS

#[get("/links")]
async fn all(auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(
        move |c| match Verifier::verify(c, &auth.email, &auth.password) {
            Ok(_link) => match LinkController::find_many(c) {
                Ok(links) => Ok(json!(links)),
                Err(e) => match e {
                    NotFound => Err(Custom(Status::NotFound, json!(e.to_string()))),
                    _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
                },
            },
            Err(e) => match e {
                NotFound => Err(Custom(Status::Unauthorized, json!("Unauthorized"))),
                _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
            },
        },
    )
    .await
}

#[get("/links/<id>")]
async fn find_by_id(auth: BasicAuth, db: DbConn, id: i32) -> Result<Value, Custom<Value>> {
    db.run(
        move |c| match Verifier::verify(c, &auth.email, &auth.password) {
            Ok(_link) => match LinkController::find_one(c, id) {
                Ok(links) => Ok(json!(links)),
                Err(e) => match e {
                    NotFound => Err(Custom(Status::NotFound, json!(e.to_string()))),
                    _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
                },
            },
            Err(e) => match e {
                NotFound => Err(Custom(Status::Unauthorized, json!("Unauthorized"))),
                _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
            },
        },
    )
    .await
}

#[post("/", format = "json", data = "<new_link>")]
async fn create_link(
    auth: BasicAuth,
    new_link: Json<NewLink>,
    db: DbConn,
) -> Result<Value, Custom<Value>> {
    db.run(
        move |c| match Verifier::verify(c, &auth.email, &auth.password) {
            Ok(_link) => match LinkController::create(c, new_link.into_inner()) {
                Ok(links) => Ok(json!(links)),
                Err(e) => match e {
                    NotFound => Err(Custom(Status::NotFound, json!(e.to_string()))),
                    _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
                },
            },
            Err(e) => match e {
                NotFound => Err(Custom(Status::Unauthorized, json!("Unauthorized"))),
                _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
            },
        },
    )
    .await
}

#[put("/links/<id>", format = "json", data = "<link>")]
async fn update_link(
    auth: BasicAuth,
    db: DbConn,
    link: Json<Link>,
    id: i32,
) -> Result<Value, Custom<Value>> {
    db.run(
        move |c| match Verifier::verify(c, &auth.email, &auth.password) {
            Ok(_link) => match LinkController::update(c, link.into_inner(), id) {
                Ok(links) => Ok(json!(links)),
                Err(e) => match e {
                    NotFound => Err(Custom(Status::NotFound, json!(e.to_string()))),
                    _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
                },
            },
            Err(e) => match e {
                NotFound => Err(Custom(Status::Unauthorized, json!("Unauthorized"))),
                _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
            },
        },
    )
    .await
}

#[delete("/links/<id>")]
async fn delete_link(auth: BasicAuth, db: DbConn, id: i32) -> Result<Value, Custom<Value>> {
    db.run(
        move |c| match Verifier::verify(c, &auth.email, &auth.password) {
            Ok(_link) => match LinkController::delete(c, id) {
                Ok(links) => Ok(json!(links)),
                Err(e) => match e {
                    NotFound => Err(Custom(Status::NotFound, json!(e.to_string()))),
                    _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
                },
            },
            Err(e) => match e {
                NotFound => Err(Custom(Status::Unauthorized, json!("Unauthorized"))),
                _ => Err(Custom(Status::InternalServerError, json!(e.to_string()))),
            },
        },
    )
    .await
}

//CATCHERS
#[catch(404)]
fn not_found() -> Value {
    json!("Not found!")
}
#[catch(401)]
fn unauthorized() -> Value {
    json!("Unauthorized")
}
#[catch(422)]
fn unprocessable_entity() -> Value {
    json!("Unprocessable entity")
}
#[catch(400)]
fn bad_request() -> Value {
    json!("Bad request")
}

// #[rocket::main]
// async fn main() {
//     let _ = rocket::build()
//         .mount(
//             "/",
//             routes![
//                 all,
//                 find_by_id,
//                 create_link,
//                 delete_link,
//                 update_link,
//                 user,
//                 delete_user,
//                 create_user,
//                 update_user,
//                 all_users,
//             ],
//         )
//         .register(
//             "/",
//             catchers![not_found, unprocessable_entity, unauthorized, bad_request],
//         )
//         .attach(DbConn::fairing())
//         .launch()
//         .await;
// }

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let allowed_origins = AllowedOrigins::some_exact(&["https://www.acme.com"]);
    // let allowed_origins = AllowedOrigins::all();

    // You can also deserialize this
    // let cors = rocket_cors::CorsOptions {
    //     allowed_origins,
    //     allowed_methods: vec![Method::Get, Method::Post, Method::Delete, Method::Put].into_iter().map(From::from).collect(),
    //     allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Access-Control-Allow-Origin"]),
    //     allow_credentials: true,
    //     ..Default::default()
    // }
    // .to_cors()?;

    let cors = rocket_cors::CorsOptions::default().to_cors()?;
    let _ = rocket::build()
        .mount(
            "/",
            routes![
                all,
                find_by_id,
                create_link,
                delete_link,
                update_link,
                user,
                delete_user,
                create_user,
                update_user,
                all_users,
            ],
        )
        .register(
            "/",
            catchers![not_found, unprocessable_entity, unauthorized, bad_request],
        )
        .attach(cors)
        .attach(DbConn::fairing())
        .launch()
        .await?;

    Ok(())
}
