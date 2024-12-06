use std::sync::Arc;

use serde::{ Deserialize, Serialize };
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SessionDTO {
    pub user_id: Arc<str>,
    pub uuid: Uuid,
}

impl SessionDTO {
    pub fn new(user_id: Arc<str>, uuid: Uuid) -> SessionDTO {
        return SessionDTO { user_id, uuid };
    }
}
