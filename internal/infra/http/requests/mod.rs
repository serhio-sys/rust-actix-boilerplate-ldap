//! Json extractor.
use core::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use actix_web::dev::{ JsonBody, Payload };
use actix_web::FromRequest;
use actix_web::HttpRequest;
use config::log::debug;
use futures::future::{ FutureExt, LocalBoxFuture };
// use futures_util::future::{LocalBoxFuture, Try};
use serde::de::DeserializeOwned;
use validator::Validate;
use error::Error;

mod error;
pub mod user_request;

#[derive(Debug)]
pub struct JsonValidator<T>(pub T);

impl<T> JsonValidator<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for JsonValidator<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for JsonValidator<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> FromRequest for JsonValidator<T> where T: DeserializeOwned + Validate + 'static {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let (limit, err, ctype) = req
            .app_data::<JsonConfig>()
            .map(|c| (c.limit, c.ehandler.clone(), c.content_type.clone()))
            .unwrap_or((4 * 1024 * 1024, None, None));

        JsonBody::new(req, payload, ctype.as_deref(), false)
            .limit(limit)
            .map(|res: Result<T, _>| {
                match res {
                    Ok(data) =>
                        data
                            .validate()
                            .map(|_| JsonValidator(data))
                            .map_err(Error::from),
                    Err(e) => Err(Error::from(e)),
                }
            })
            .map(move |res| {
                match res {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        debug!(
                            "Failed to deserialize Json from payload. \
                         Request path: {}",
                            req2.path()
                        );
                        if let Some(err) = err {
                            Err((*err)(e, &req2))
                        } else {
                            Err(e.into())
                        }
                    }
                }
            })
            .boxed_local()
    }
}

type ErrHandler = Arc<dyn (Fn(Error, &HttpRequest) -> actix_web::Error) + Send + Sync>;

#[derive(Clone)]
pub struct JsonConfig {
    limit: usize,
    ehandler: Option<ErrHandler>,
    content_type: Option<Arc<dyn (Fn(mime::Mime) -> bool) + Send + Sync>>,
}

impl JsonConfig {
    /// Change max size of payload. By default max size is 32Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
        where F: Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync + 'static
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    /// Set predicate for allowed content types
    pub fn content_type<F>(mut self, predicate: F) -> Self
        where F: Fn(mime::Mime) -> bool + Send + Sync + 'static
    {
        self.content_type = Some(Arc::new(predicate));
        self
    }
}

impl Default for JsonConfig {
    fn default() -> Self {
        JsonConfig {
            limit: 4 * 1024 * 1024,
            ehandler: None,
            content_type: None,
        }
    }
}
