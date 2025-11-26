use deadpool_redis::Pool;
use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static REDIS_POOL: Lazy<RwLock<Option<Pool>>> = Lazy::new(|| RwLock::new(None));
