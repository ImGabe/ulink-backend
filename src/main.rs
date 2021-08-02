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
use rocket::http::Header;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{catch, get, launch, options, post, routes, Request, Responder, Response};
use rocket::{
    catchers,
    fairing::{Fairing, Info, Kind},
    http::Status,
};
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

        if result {
            break id;
        }
    };

    let location = format!("/{}", &id);

    if data.duration.is_none() {
        return Created::new(location).body(Json(ShorterURL::new(id, data.url.clone(), None)));
    }

    conn.expire::<&str, usize>(
        &format!("{}::{}", REDIS_KEY_PREFIX, id),
        data.duration.expect("Faild Duration"),
    )
    .await
    .expect("RedisExpireError");

    Created::new(location).body(Json(ShorterURL::new(id, data.url.clone(), data.duration)))
}

#[derive(Responder)]
enum AccessResponse {
    Found(Redirect),
    NotFound(Redirect),
}

#[get("/<id>")]
async fn access(id: &str, mut conn: RedisConnection<'_>) -> AccessResponse {
    let key = format!("{}::{}", REDIS_KEY_PREFIX, id);

    match conn.get::<String, String>(key).await {
        Ok(url) => AccessResponse::Found(Redirect::to(url)),
        Err(_) => AccessResponse::NotFound(Redirect::to("https://imgabe.github.io/ulink/")),
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
        let origin = env::var("ORIGIN_ADDRESS").expect("No ORIGIN_ADDRESS found");

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

#[catch(500)]
fn internal_error() -> &'static str {
    "Whoops! Looks like we messed up."
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("I couldn't find '{}'. Try something else?", req.uri())
}

#[catch(default)]
fn default(status: Status, req: &Request) -> String {
    format!("{} ({})", status, req.uri())
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").expect("NoRedisURL");

    rocket::build()
        .attach(Cors)
        .manage(db::pool(&redis_url).await)
        .mount("/", routes![index, access, shorten, cors])
        .register("/", catchers![internal_error, not_found, default])
}
