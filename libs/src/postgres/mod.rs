use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::sync::RwLock;
pub static PG_POOL: Lazy<RwLock<Option<PgPool>>> = Lazy::new(|| RwLock::new(None));
