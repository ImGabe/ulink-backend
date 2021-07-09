#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate redis;

mod models;

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::Redirect;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;

use nanoid::nanoid;
use redis::Commands;
use serde_json::Value;

use models::{Link, LinkStatus};

fn connect() -> redis::Connection {
    redis::Client::open("redis://127.0.0.1:6379")
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}
#[derive(Debug)]
struct ApiResponse {
    json: Value,
    status: Status,
}

impl<'r, 'a> Responder<'r, 'r> for ApiResponse {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[get("/")]
fn index() -> Value {
    let mut conn = connect();
    let ids: Vec<String> = conn.keys("*").expect("failed to get all keys");
    let mut links: Vec<Link> = Vec::new();

    for id in ids {
        let url: String = conn.get(&id).expect("failed to execute GET");
        let duration: usize = conn.ttl(&id).expect("failed to execute TTL");

        links.push(Link { id, url, duration });
    }

    serde_json::json!(links)
}

#[get("/<id>")]
fn redirect(id: &str) -> Redirect {
    let mut conn = connect();
    let url_redirect: String = conn.get(id).expect("failed to execute GET");

    Redirect::to(url_redirect)
}

#[post("/", format = "json", data = "<link>")]
fn new(link: Json<Link>) -> ApiResponse {
    let id = nanoid!(5);
    let mut conn = connect();

    let _: () = conn
        .set::<&str, &str, ()>(&id, &link.url)
        .expect("failed to execute SET");
    let _: () = conn
        .expire(&id, link.duration)
        .expect("faile to execute EXPIRE");

    ApiResponse {
        json: serde_json::json!(LinkStatus {
            id: id,
            url: link.url.clone(),
            duration: link.duration
        }),
        status: Status::Created,
    }
}

#[put("/", format = "json", data = "<link>")]
fn edit_url(link: Json<Link>) -> ApiResponse {
    let mut conn = connect();

    let exist: bool = conn.exists(&link.id).expect("faile to execute EXISTS");
    if !exist {
        return ApiResponse {
            json: serde_json::json!(LinkStatus {
                id: String::new(),
                url: String::new(),
                duration: 0
            }),
            status: Status::NotFound,
        };
    }

    let _: () = conn
        .getset(&link.id, &link.url)
        .expect("faile to execute GETSET");
    let _: () = conn
        .expire(&link.id, link.duration)
        .expect("faile to execute EXPIRE");

    ApiResponse {
        json: serde_json::json!(LinkStatus {
            id: link.id.clone(),
            url: link.url.clone(),
            duration: link.duration
        }),
        status: Status::Accepted,
    }
}

#[delete("/", format = "json", data = "<link>")]
fn delete_url(link: Json<Link>) -> ApiResponse {
    let mut conn = connect();

    let exist: bool = conn.exists(&link.id).expect("faile to execute EXISTS");
    if !exist {
        return ApiResponse {
            json: serde_json::json!(LinkStatus {
                id: String::new(),
                url: String::new(),
                duration: 0
            }),
            status: Status::NotFound,
        };
    }

    let url: String = conn.get(&link.id).expect("failed to execute SET");
    let _: () = conn.del(&link.id).expect("failed to execute DEL");

    ApiResponse {
        json: serde_json::json!(LinkStatus {
            id: link.id.clone(),
            url: url,
            duration: link.duration
        }),
        status: Status::Ok,
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, new, redirect, edit_url, delete_url])
}
