use std::sync::Arc;

use serde::Serialize;

use crate::infra::{
    database::user_repository::User,
    http::{ middlewares::Userable, resources::user_resource::UserResponse },
};

#[derive(Clone, PartialEq, Serialize)]
pub struct UserDTO {
    pub uid: Arc<str>,
    pub name: Arc<str>,
    pub email: Arc<str>,
}

#[derive(Clone, Serialize)]
pub struct AuthenticatedUserDTO {
    pub user: UserResponse,
    pub token: Arc<str>,
}

impl UserDTO {
    pub(crate) fn model_to_dto(user: User) -> UserDTO {
        return UserDTO {
            uid: user.uid,
            name: user.sn,
            email: user.cn,
        };
    }

    pub(crate) fn models_to_dto(users: Vec<User>) -> Vec<UserDTO> {
        let mut users_dto: Vec<UserDTO> = Vec::new();
        for user in users {
            users_dto.push(UserDTO::model_to_dto(user));
        }
        return users_dto;
    }

    pub fn dto_to_model(&self) -> User {
        return User {
            uid: self.uid.clone(),
            sn: self.name.clone(),
            cn: self.email.clone(),
        };
    }
}

impl Userable for UserDTO {
    fn get_user_id(&self) -> Arc<str> {
        return self.email.clone();
    }
}
