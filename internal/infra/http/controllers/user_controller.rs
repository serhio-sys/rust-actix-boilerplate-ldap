use std::sync::Arc;

use actix_web::{ web, HttpMessage, HttpRequest, HttpResponse, Responder };

use crate::{
    infra::{
        domain::user::UserDTO,
        http::resources::{ user_resource::UserResponse, BasedListResponse, ErrorResponse },
    },
    services::user_service::UserService,
};

#[derive(Clone)]
pub struct UserController {
    user_service: Arc<UserService>,
}

impl UserController {
    pub fn new(user_service: Arc<UserService>) -> UserController {
        return UserController { user_service };
    }

    async fn find_all(&self) -> impl Responder {
        match self.user_service.find_all().await {
            Ok(users) => {
                let response = BasedListResponse {
                    data: UserResponse::dtos_to_response(users),
                    total: 0,
                    page: 0,
                };
                return HttpResponse::Ok().json(response);
            }
            Err(e) => {
                return HttpResponse::BadRequest().json(
                    ErrorResponse::new_error(Some(e.to_string()))
                );
            }
        }
    }

    async fn find_me(&self, request: HttpRequest) -> impl Responder {
        if let Some(user) = request.extensions_mut().get::<UserDTO>() {
            return HttpResponse::Ok().json(UserResponse::dto_to_response(user));
        }
        return HttpResponse::BadRequest().json("Something went wrong");
    }
}

// HANDLERS USER ROUTE
pub async fn find_me(
    user_controller: web::Data<UserController>,
    request: HttpRequest
) -> impl Responder {
    return user_controller.find_me(request).await;
}

pub async fn find_all(user_controller: web::Data<UserController>) -> impl Responder {
    return user_controller.find_all().await;
}
