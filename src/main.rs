extern crate dotenv;

mod consts;
mod db;
mod models;

#[cfg(test)]
mod test;

use bb8_redis::redis::AsyncCommands;
use db::RedisConnection;
use dotenv::dotenv;
use models::{NewShorterURL, ShorterURL};
use nanoid::nanoid;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, launch, options, post, routes, Request, Responder, Response};
use rocket::{
    http::{ContentType, Method},
    response::status::Created,
};
use std::env;

const REDIS_KEY_PREFIX: &str = "microshort::ids";

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[options("/")]
async fn cors() {}

#[post("/", format = "json", data = "<data>")]
async fn shorten(
    data: Json<NewShorterURL>,
    mut conn: RedisConnection<'_>,
) -> Created<Json<ShorterURL>> {
    let id = loop {
        let id = nanoid!(4, &consts::ALPHANUMERIC);
        let key = format!("{}::{}", REDIS_KEY_PREFIX, id);
        let result = conn.set_nx(&key, &data.url).await.expect("RedisSetNXError");

        conn.expire::<&str, usize>(&key, data.duration)
            .await
            .expect("RedisExpireError");

        if result {
            break id;
        }
    };

    let location = format!("/{}", &id);
    Created::new(location).body(Json(ShorterURL::new(id, data.url.clone(), data.duration)))
}

#[derive(Responder)]
enum AccessResponse {
    Found(Redirect),
    #[response(status = 404)]
    NotFound(()),
}

#[get("/<id>")]
async fn access(id: &str, mut conn: RedisConnection<'_>) -> AccessResponse {
    let key = format!("{}::{}", REDIS_KEY_PREFIX, id);

    match conn.get::<String, String>(key).await {
        Ok(url) => AccessResponse::Found(Redirect::to(url)),
        Err(_) => AccessResponse::NotFound(()),
    }
}

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "My Custom Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let origin = match env::var("ORIGIN_ADDRESS") {
            Ok(val) => val,
            Err(e) => panic!("{}", e),
        };

        if req.method() == Method::Options || res.content_type() == Some(ContentType::JSON) {
            res.set_header(Header::new("Access-Control-Allow-Origin", origin));
            res.set_header(Header::new(
                "Access-Control-Allow-Methods",
                "POST, GET, OPTIONS",
            ));
            res.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
            res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        }

        if req.method() == Method::Options {
            res.set_header(ContentType::Plain);
        }
    }
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    rocket::build()
        .attach(Cors)
        .manage(db::pool().await)
        .mount("/", routes![index, access, shorten, cors])
}
