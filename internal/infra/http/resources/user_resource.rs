use std::sync::Arc;

use serde::Serialize;

use crate::infra::{ database::user_repository::User, domain::user::UserDTO };

#[derive(Clone, Serialize)]
pub struct UserResponse {
    pub uid: Arc<str>,
    pub name: Arc<str>,
    pub email: Arc<str>,
}

impl UserResponse {
    pub fn dto_to_response(dto: &UserDTO) -> Self {
        return UserResponse {
            uid: dto.uid.clone(),
            name: dto.name.clone(),
            email: dto.email.clone(),
        };
    }

    pub fn user_to_response(dto: &User) -> Self {
        return UserResponse {
            uid: dto.uid.clone(),
            name: dto.sn.clone(),
            email: dto.cn.clone(),
        };
    }

    pub fn dtos_to_response(dtos: Vec<UserDTO>) -> Vec<Self> {
        let mut response_objects: Vec<Self> = Vec::new();
        for dto in dtos {
            response_objects.push(Self::dto_to_response(&dto));
        }
        return response_objects;
    }
}
