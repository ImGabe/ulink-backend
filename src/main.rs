mod consts;
mod db;
mod models;

use bb8_redis::redis::AsyncCommands;
use db::RedisConnection;
use models::{NewShorterURL, ShorterURL};
use nanoid::nanoid;
use rocket::response::status::Created;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, launch, post, routes, Responder};

const REDIS_KEY_PREFIX: &str = "microshort::ids";

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

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
    Created::new(location).body(Json(ShorterURL {
        id,
        url: data.url.clone(),
        duration: data.duration,
    }))
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

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .manage(db::pool().await)
        .mount("/", routes![index, access, shorten])
}
