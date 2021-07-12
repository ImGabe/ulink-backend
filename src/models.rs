use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ShorterURL {
    pub id: String,
    pub url: String,
    pub duration: usize,
}

#[derive(Deserialize)]
pub struct NewShorterURL {
    pub url: String,
    pub duration: usize,
}
