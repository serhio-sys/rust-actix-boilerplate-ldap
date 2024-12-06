use core::error;
use std::sync::Arc;
use async_trait::async_trait;
use config::log::error;
use thiserror::Error;

use crate::infra::{
    database::user_repository::UserRepository,
    domain::user::UserDTO,
    http::middlewares::Findable,
};

pub struct UserService {
    user_repository: Arc<UserRepository>,
}

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("Database error: {0}")] DieselError(diesel::result::Error),
    #[error("{0}")] ServiceError(Box<dyn error::Error + Send + Sync + 'static>),
}

#[async_trait]
impl Findable<UserDTO> for UserService {
    async fn find_by_id(
        &self,
        user_id: Arc<str>
    ) -> Result<UserDTO, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let user = self.find_user_by_id(user_id).await?;
        return Ok(user);
    }
}

impl UserService {
    pub fn new(user_repository: Arc<UserRepository>) -> Arc<UserService> {
        return Arc::from(UserService {
            user_repository,
        });
    }

    pub async fn find_all(
        &self
    ) -> Result<Vec<UserDTO>, Box<dyn error::Error + Send + Sync + 'static>> {
        let users = self.user_repository.find_all().await?;
        return Ok(UserDTO::models_to_dto(users));
    }

    pub async fn find_user_by_id(
        &self,
        user_id: Arc<str>
    ) -> Result<UserDTO, Box<dyn error::Error + Send + Sync + 'static>> {
        let user = self.user_repository.find_by_id(user_id).await?;
        return Ok(UserDTO::model_to_dto(user));
    }
}
