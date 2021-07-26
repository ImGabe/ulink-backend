use rocket::http::Status;
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct NewShorterURL {
    url: String,
    duration: usize,
}

impl NewShorterURL {
    fn new(url: String, duration: usize) -> Self {
        Self { url, duration }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ShorterURL {
    id: String,
    url: String,
    duration: usize,
}

#[rocket::async_test]
async fn test_index() {
    use rocket::local::asynchronous::Client;

    let client = Client::tracked(super::rocket().await).await.unwrap();
    let req = client.get("/").dispatch().await;

    assert_eq!(req.status(), Status::Ok);
    assert_eq!(req.into_string().await.unwrap(), "Hello, world!");
}

#[rocket::async_test]
async fn test_shorten() {
    use rocket::local::asynchronous::Client;

    let client = Client::tracked(super::rocket().await).await.unwrap();
    let req = client
        .post("/")
        .json(&NewShorterURL::new("https://github.com".to_string(), 100))
        .dispatch()
        .await;

    assert_eq!(req.status(), Status::Created);
}

#[rocket::async_test]
async fn test_access() {
    use rocket::local::asynchronous::Client;

    let client = Client::tracked(super::rocket().await).await.unwrap();
    let req_post = client
        .post("/")
        .json(&NewShorterURL::new("https://github.com".to_string(), 100))
        .dispatch()
        .await;

    let location = req_post.headers().get_one("Location").unwrap();

    let req_get = client.get(location).dispatch().await;
    let redirect_to = req_get.headers().get_one("Location").unwrap();

    assert_eq!(req_get.status(), Status::SeeOther);
    assert_eq!(redirect_to, "https://github.com");
}
