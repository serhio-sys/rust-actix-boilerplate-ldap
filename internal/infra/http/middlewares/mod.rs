use std::sync::Arc;

use serde::Serialize;
use async_trait::async_trait;

pub mod auth_middleware;
pub mod is_owner_middleware;
pub mod path_object_middleware;

pub trait Userable {
    fn get_user_id(&self) -> Arc<str>;
}

#[async_trait]
pub trait Findable<T> where T: Serialize {
    async fn find_by_id(
        &self,
        id: Arc<str>
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
}
