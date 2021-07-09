use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Link {
    pub id: String,
    pub url: String,
    pub duration: usize,
}

#[derive(Serialize, Deserialize)]
pub struct LinkStatus {
    pub id: String,
    pub url: String,
    pub duration: usize,
}