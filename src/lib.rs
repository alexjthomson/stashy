//! # Stashy
//! Stashing made simple.

mod local;

#[cfg(feature = "redis")]
mod redis;

pub use local::LocalStash;
#[cfg(feature = "redis")]
pub use redis::{
    RedisCredentials,
    RedisError,
    RedisStash,
};

use thiserror::Error;

/// An error that can occur when interacting with a [`Stash`].
#[derive(Debug, Error)]
pub enum StashError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Backend error: {0}")]
    BackendError(#[from] Box<dyn core::error::Error + Send + Sync>),
}

impl StashError {
    /// Returns a new [`StashError::BackendError`] using the provided `error` as
    /// the cause.
    #[inline]
    #[must_use]
    pub fn backend<E>(error: E) -> Self
    where
        E: core::error::Error + Send + Sync + 'static
    {
        Self::BackendError(Box::new(error))
    }
}

#[async_trait::async_trait]
pub trait Stash: Send + Sync {
    /// Gets a value from the stash.
    async fn fetch<K>(
        &self,
        key: K,
    ) -> Result<Option<String>, StashError>
    where
        K: AsRef<str> + Send + Sync,
    ;

    /// Sets a value within the stash and returns the previous value (if there
    /// was one).
    /// 
    /// # Naming Convention
    /// When inserting a `key` into the stash, you must only use alphanumberic
    /// characters, underscores, and colons as delimitors.
    /// 
    /// For example: `user:123:name`, `user:123:email`, `session:f05a29`, etc.
    async fn stash<K, V>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<String>, StashError>
    where
        K: Into<String> + Send + Sync,
        V: Into<String> + Send + Sync,
    ;

    /// Deletes an entry from the stash and returns the previous value (if there
    /// was one).
    async fn delete<K>(
        &self,
        key: K,
    ) -> Result<Option<String>, StashError>
    where
        K: AsRef<str> + Send + Sync,
    ;

    /// Validates a stash key.
    /// 
    /// This should be called internally by types that implement [`Stash`].
    #[inline]
    fn validate_key(
        key: &str,
    ) -> Result<(), StashError> {
        if key.is_empty() {
            return Err(StashError::InvalidKey("Key must not be empty".into()));
        }
        if key.starts_with(':') || key.ends_with(':') {
            return Err(StashError::InvalidKey("Key must not start or end with ':'".into()));
        }
        if key.split(':').any(|segment| segment.is_empty()) {
            return Err(StashError::InvalidKey("Key must not contain empty segments".into()));
        }
        if !key.chars().all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '_') {
            return Err(StashError::InvalidKey("Key contains invalid characters".into()));
        }
        Ok(())
    }
}