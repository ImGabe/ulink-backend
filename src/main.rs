#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate redis;

use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, launch, routes};

use nanoid::nanoid;
use redis::Commands;
use serde_json::Value;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
struct Link {
    id: String,
    url: String,
    duration: usize,
}

#[derive(Serialize, Deserialize)]
struct LinkStatus {
    id: String,
    url: String,
    duration: usize,
}

fn connect() -> redis::Connection {
    redis::Client::open("redis://127.0.0.1:6379")
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

#[get("/")]
fn index() -> Value {
    let mut conn = connect();
    let ids: Vec<String> = conn.keys("*").unwrap();
    let mut links: Vec<Link> = Vec::new();

    for id in ids {
        let url: String = conn.get(id.clone()).expect("failed to execute GET");
        let duration: usize = conn.ttl(id.clone()).unwrap();

        links.push(Link { id, url, duration });
    }

    serde_json::json!(links)
}

#[get("/<id>")]
fn redirect(id: String) -> Redirect {
    let mut conn = connect();
    let url_redirect: String = conn.get(id).expect("failed to execute GET");

    Redirect::to(url_redirect)
}

#[post("/", format = "json", data = "<link>")]
fn new(link: Json<Link>) -> Value {
    let id = nanoid!(5);
    let mut conn = connect();

    let _: () = conn.set::<&str, &str, ()>(&id, &link.url).unwrap();
    let _: () = conn.expire(&id, link.duration).unwrap();

    serde_json::json!(LinkStatus { id: id, url: link.url.clone(), duration: link.duration })
}

#[put("/", format = "json", data = "<link>")]
fn edit_url(link: Json<Link>) -> Value {
    let mut conn = connect();
    
    let _: () = conn.getset(link.id.clone(), link.url.clone()).unwrap();
    let _: () = conn.expire(link.id.clone(), link.duration).unwrap();

    serde_json::json!(LinkStatus { id: link.id.clone(), url: link.url.clone(), duration: link.duration })
}

#[delete("/", format = "json", data = "<link>")]
fn delete_url(link: Json<Link>) -> Value {
    let mut conn = connect();

    let url: String = conn.get(link.id.clone()).unwrap();
    let _: () = conn.del(link.id.clone()).unwrap();

    serde_json::json!(LinkStatus { id: link.id.clone(), url: url, duration: link.duration })
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, new, redirect, edit_url, delete_url])
}
