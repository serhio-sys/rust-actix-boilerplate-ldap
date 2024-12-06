use std::sync::Arc;

use config::CONFIGURATION;
use ldap3::{ Ldap, LdapError, SearchEntry };
use tokio::sync::RwLock;
use core::error;
pub struct User {
    pub cn: Arc<str>,
    pub uid: Arc<str>,
    pub sn: Arc<str>,
}

pub struct UserRepository {
    pub ldap: Arc<RwLock<Ldap>>,
}

impl UserRepository {
    pub fn new(ldap: Arc<RwLock<Ldap>>) -> Arc<UserRepository> {
        return Arc::new(UserRepository { ldap });
    }

    pub async fn find_all(&self) -> Result<Vec<User>, LdapError> {
        let result = self.ldap
            .write().await
            .search(
                &CONFIGURATION.ldap_auth_base_dn,
                ldap3::Scope::Subtree,
                "(objectClass=inetOrgPerson)",
                vec!["dn", "cn", "sn", "uid"]
            ).await?;

        let entries = result.success()?.0;
        let mut parsed_entries = Vec::new();
        for entry in entries {
            let en = SearchEntry::construct(entry).attrs;
            parsed_entries.push(User {
                cn: Arc::from(en.get("cn").unwrap().get(0).unwrap().as_str()),
                uid: Arc::from(en.get("uid").unwrap().get(0).unwrap().as_str()),
                sn: Arc::from(en.get("sn").unwrap().get(0).unwrap().as_str()),
            });
        }
        return Ok(parsed_entries);
    }

    pub async fn find_by_id(
        &self,
        user_id: Arc<str>
    ) -> Result<User, Box<dyn error::Error + Send + Sync + 'static>> {
        let result = self.ldap
            .write().await
            .search(
                &format!("cn={},{}", user_id, CONFIGURATION.ldap_auth_base_dn),
                ldap3::Scope::Subtree,
                "(objectClass=inetOrgPerson)",
                vec!["dn", "cn", "sn", "uid"]
            ).await?;
        let (entries, _) = result.success()?;
        if entries.is_empty() {
            return Err(Box::from("There is no one user was found"));
        }
        if entries.len() > 1 {
            return Err(Box::from("Multiply users was found"));
        }
        let user_dn = SearchEntry::construct(entries[0].clone());
        let user = User {
            cn: Arc::from(user_dn.attrs.get("cn").unwrap().get(0).unwrap().as_str()),
            uid: Arc::from(user_dn.attrs.get("uid").unwrap().get(0).unwrap().as_str()),
            sn: Arc::from(user_dn.attrs.get("sn").unwrap().get(0).unwrap().as_str()),
        };
        return Ok(user);
    }
}
