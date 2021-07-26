use std::ops::{Deref, DerefMut};

use bb8_redis::RedisConnectionManager;
use rocket::{
    http,
    outcome::try_outcome,
    request::{FromRequest, Outcome, Request},
    State,
};

const REDIS_URL: &str = "redis://0.0.0.0:6379";

type Pool = bb8::Pool<RedisConnectionManager>;

pub async fn pool() -> Pool {
    let manager = RedisConnectionManager::new(REDIS_URL).unwrap();
    Pool::builder().build(manager).await.expect("RedisPoolFail")
}

pub struct RedisConnection<'a>(pub bb8::PooledConnection<'a, RedisConnectionManager>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RedisConnection<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = try_outcome!(request.guard::<&State<Pool>>().await);

        match pool.get().await {
            Ok(conn) => Outcome::Success(RedisConnection(conn)),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}

impl<'a> Deref for RedisConnection<'a> {
    type Target = bb8::PooledConnection<'a, RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RedisConnection<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
