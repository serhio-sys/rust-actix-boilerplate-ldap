use std::sync::{ Arc, RwLock };

use diesel::{
    prelude::{ Insertable, Queryable },
    query_dsl::methods::FilterDsl,
    r2d2::{ ConnectionManager, Pool, PooledConnection },
    ExpressionMethods,
    PgConnection,
    RunQueryDsl,
    Selectable,
};
use uuid::Uuid;

use crate::infra::domain::session::SessionDTO;

diesel::table! {
    sessions (user_id, uuid) {
        user_id -> Text,
        uuid -> Uuid,
    }
}

#[derive(Selectable, Insertable, Queryable, Debug)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub user_id: String,
    pub uuid: Uuid,
}

#[derive(Clone)]
pub struct SessionRepository {
    pub pool: Arc<RwLock<Pool<ConnectionManager<PgConnection>>>>,
}

impl SessionRepository {
    pub fn new(pool: Arc<RwLock<Pool<ConnectionManager<PgConnection>>>>) -> Arc<SessionRepository> {
        return Arc::new(SessionRepository { pool });
    }

    fn get_connection(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.write().unwrap().get().expect("Failed to get a connection")
    }

    pub fn save(&self, session: SessionDTO) -> Result<Session, diesel::result::Error> {
        use self::sessions::dsl::*;
        let session_model = Session { user_id: session.user_id.to_string(), uuid: session.uuid };
        let result = diesel
            ::insert_into(sessions)
            .values(&session_model)
            .get_result::<Session>(&mut self.get_connection())?;
        return Ok(result);
    }

    pub fn exists(&self, session: SessionDTO) -> Result<bool, diesel::result::Error> {
        use self::sessions::dsl::*;
        use diesel::dsl::exists;
        let exists = diesel
            ::select(
                exists(
                    sessions
                        .filter(user_id.eq(&session.user_id.to_string()))
                        .filter(uuid.eq(&session.uuid))
                )
            )
            .get_result::<bool>(&mut self.get_connection())?;
        return Ok(exists);
    }

    pub fn delete(&self, session: SessionDTO) -> Result<usize, diesel::result::Error> {
        use self::sessions::dsl::*;
        let result = diesel
            ::delete(
                sessions
                    .filter(user_id.eq(&session.user_id.to_string()))
                    .filter(uuid.eq(&session.uuid))
            )
            .execute(&mut self.get_connection());
        return result;
    }

    pub fn delete_by_user_id(&self, id: String) -> Result<usize, diesel::result::Error> {
        use self::sessions::dsl::*;
        let result = diesel
            ::delete(sessions.filter(user_id.eq(id)))
            .execute(&mut self.get_connection());
        return result;
    }
}
