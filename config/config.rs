use dotenvy::{ dotenv, var };
use lazy_static::lazy_static;

pub mod logger;
pub use log;

lazy_static! {
    pub static ref CONFIGURATION: Configuration = {
        return get_configuration();
    };
}

pub struct Configuration {
    pub database_name: String,
    pub database_user: String,
    pub database_password: String,
    pub database_host: String,
    pub migration_location: String,
    pub migration_version: String,
    pub file_storage_location: String,
    pub jwt_ttl: u64,
    pub jwt_secret: String,
    pub ldap_url: String,
    pub ldap_auth_base_dn: String,
}

fn get_configuration() -> Configuration {
    if let Err(exc) = dotenv() {
        log::error!("Error in loading .env file - [{}]", exc.to_string());
    }
    return Configuration {
        database_name: get_var("DATABASE_NAME"),
        database_host: get_var("DATABASE_HOST"),
        database_user: get_var("DATABASE_USER"),
        database_password: get_var("DATABASE_PASSWORD"),
        migration_location: get_var("MIGRATION_LOCATION"),
        // 2024-09-21-122416 - example of migration verison.
        // latest - for running migration to the last one in migrations folder.
        migration_version: get_var_or_default("MIGRATE_TO", "latest"),
        file_storage_location: get_var_or_default("FILE_STORAGE_LOCATION", "file_storage"),
        jwt_ttl: 72 * 3600,
        jwt_secret: get_var_or_default("JWT_SECRET", "1234567890"),

        // ldap
        ldap_url: get_var("LDAP_URL"),
        ldap_auth_base_dn: get_var("LDAP_AUTH_BASE_DN"),
    };
}

#[allow(dead_code)]
fn get_var_or_default(key: &str, def_value: &str) -> String {
    let value = var(key);
    if let Ok(unwrapped_value) = value {
        return unwrapped_value;
    }
    return def_value.to_string();
}

fn get_var(key: &str) -> String {
    let value = var(key);
    if let Ok(unwrapped_value) = value {
        return unwrapped_value;
    } else {
        panic!(
            "Error in getting value from .env by key[{}]\n[{}]",
            key,
            value.unwrap_err().to_string()
        );
    }
}
