use core::error;
use std::{ sync::Arc, time::{ Duration, SystemTime, UNIX_EPOCH } };

use config::CONFIGURATION;
use ldap3::{ Ldap, LdapError, SearchEntry };
use pwhash::bcrypt::{ self, BcryptSetup };
use jsonwebtoken::{ EncodingKey, Header };
use serde::{ Deserialize, Serialize };
use thiserror::Error;
use uuid::Uuid;

use crate::infra::{
    database::{ session_repository::{ Session, SessionRepository }, user_repository::User },
    domain::{ session::SessionDTO, user::AuthenticatedUserDTO },
    http::{ requests::user_request::AuthRequest, resources::user_resource::UserResponse },
};

#[derive(Serialize, Clone, Deserialize)]
pub struct Claims {
    pub user_id: Arc<str>,
    pub uuid: Uuid,
    pub exp: usize,
}

pub struct AuthService {
    ldap: Arc<tokio::sync::RwLock<Ldap>>,
    session_repository: Arc<SessionRepository>,
}

#[derive(Error, Debug)]
pub enum AuthServiceError {
    #[error("{0}")] DieselError(diesel::result::Error),
    #[error("{0}")] ArgonError(pwhash::error::Error),
    #[error("{0}")] JWTError(jsonwebtoken::errors::Error),
    #[error("{0}")] LDAPError(LdapError),
    #[error("{0}")] ServiceError(Box<dyn error::Error + Send + Sync + 'static>),
}

impl AuthService {
    pub fn new(
        ldap: Arc<tokio::sync::RwLock<Ldap>>,
        session_repository: Arc<SessionRepository>
    ) -> Arc<AuthService> {
        return Arc::new(AuthService {
            ldap,
            session_repository,
        });
    }

    pub async fn login(
        &self,
        request_user: AuthRequest
    ) -> Result<AuthenticatedUserDTO, AuthServiceError> {
        let result = self.ldap
            .write().await
            .search(
                &format!("cn={},{}", request_user.email, CONFIGURATION.ldap_auth_base_dn),
                ldap3::Scope::Subtree,
                "(objectClass=inetOrgPerson)",
                vec!["dn", "cn", "sn", "uid"]
            ).await
            .map_err(AuthServiceError::LDAPError)?;
        let (entries, _) = result.success().map_err(AuthServiceError::LDAPError)?;
        if entries.is_empty() {
            return Err(AuthServiceError::ServiceError(Box::from("There is no one user was found")));
        }
        if entries.len() > 1 {
            return Err(AuthServiceError::ServiceError(Box::from("Multiply users was found")));
        }
        let user_dn = SearchEntry::construct(entries[0].clone());
        let bind_result = self.ldap
            .write().await
            .simple_bind(&user_dn.dn, &request_user.password).await
            .map_err(AuthServiceError::LDAPError)?;
        if bind_result.success().is_ok() {
            let user = User {
                cn: Arc::from(user_dn.attrs.get("cn").unwrap().get(0).unwrap().as_str()),
                uid: Arc::from(user_dn.attrs.get("uid").unwrap().get(0).unwrap().as_str()),
                sn: Arc::from(user_dn.attrs.get("sn").unwrap().get(0).unwrap().to_owned()),
            };
            let token = self.generate_jwt(user.cn.clone())?;
            return Ok(AuthenticatedUserDTO {
                user: UserResponse::user_to_response(&user),
                token: token,
            });
        }
        return Err(AuthServiceError::ServiceError(Box::from("Auth error")));
    }

    pub fn logout(&self, session: SessionDTO) -> Result<(), AuthServiceError> {
        self.session_repository.delete(session).map_err(AuthServiceError::DieselError)?;
        return Ok(());
    }

    pub async fn check(&self, session: Arc<Claims>) -> bool {
        let res = self.session_repository.exists(SessionDTO {
            user_id: session.user_id.clone(),
            uuid: session.uuid,
        });
        if res.is_ok() {
            return res.unwrap();
        }
        return false;
    }

    fn generate_jwt(&self, user_id: Arc<str>) -> Result<Arc<str>, AuthServiceError> {
        let session = SessionDTO { user_id, uuid: Uuid::new_v4() };
        let saved_session: Session = self.session_repository
            .save(session)
            .map_err(AuthServiceError::DieselError)?;
        let claims = Claims {
            user_id: Arc::from(saved_session.user_id.as_str()),
            uuid: saved_session.uuid,
            exp: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize) +
            (Duration::from_secs(CONFIGURATION.jwt_ttl).as_secs() as usize),
        };
        let token = jsonwebtoken
            ::encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(CONFIGURATION.jwt_secret.as_ref())
            )
            .map_err(AuthServiceError::JWTError)?;
        return Ok(Arc::from(token.as_str()));
    }
}

pub fn hash_user_password(password: &str) -> Result<String, pwhash::error::Error> {
    return bcrypt::hash_with(
        BcryptSetup {
            variant: Some(bcrypt::BcryptVariant::V2b),
            cost: Some(5),
            ..Default::default()
        },
        password
    );
}
