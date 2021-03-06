use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ShorterURL {
    pub id: String,
    pub url: String,
    pub duration: Option<usize>,
}

impl ShorterURL {
    pub fn new(id: String, url: String, duration: Option<usize>) -> Self {
        Self { id, url, duration }
    }
}

#[derive(Deserialize)]
pub struct NewShorterURL {
    pub url: String,
    pub duration: Option<usize>,
}
