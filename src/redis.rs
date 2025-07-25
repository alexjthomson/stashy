use std::ops::{Deref, DerefMut};

use redis::{aio::MultiplexedConnection, AsyncCommands, Client};

use crate::{Stash, StashError};

pub use redis::RedisError;

impl From<RedisError> for StashError {
    fn from(error: RedisError) -> Self {
        Self::backend(error)
    }
}

/// Contains Redis user credentials.
/// 
/// This is typically used when calling [`RedisStash::connect`].
pub struct RedisCredentials {
    pub username: String,
    pub password: String,
}

/// [`Stash`] connected to a Redis database.
#[derive(Clone)]
pub struct RedisStash(MultiplexedConnection);

impl RedisStash {
    /// Connects to a Redis server and returns a new [`RedisStash`].
    pub async fn connect(
        host: &str,
        port: u16,
        credentials: Option<RedisCredentials>,
        database_index: Option<u32>,
    ) -> Result<Self, RedisError> {
        let url = match (credentials, database_index) {
            (Some(credentials), Some(database_index)) => format!(
                "redis://{}:{}@{}:{}/{}",
                credentials.username,
                credentials.password,
                host,
                port,
                database_index,
            ),
            (Some(credentials), None) => format!(
                "redis://{}:{}@{}:{}",
                credentials.username,
                credentials.password,
                host,
                port,
            ),
            (None, Some(database_index)) => format!(
                "redis://{}:{}/{}",
                host,
                port,
                database_index,
            ),
            (None, None) => format!(
                "redis://{}:{}/0",
                host,
                port,
            ),
        };
        Self::connect_with_string(url).await
    }

    /// Connects to a Redis server and returns a new [`RedisStash`].
    pub async fn connect_with_string<T: Into<String>>(connection_string: T) -> Result<Self, RedisError> {
        let url: String = connection_string.into();
        let client = Client::open(url)?;
        let connection = client.get_multiplexed_async_connection().await?;
        Ok(Self(connection))
    }
}

impl Deref for RedisStash {
    type Target = MultiplexedConnection;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RedisStash {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait::async_trait]
impl Stash for RedisStash {
    async fn fetch<K>(
        &self,
        key: K,
    ) -> Result<Option<String>, StashError>
    where
        K: AsRef<str> + Send + Sync,
    {
        Ok(
            self.0
                .clone()
                .get(key.as_ref())
                .await?
        )
    }

    async fn stash<K, V>(
        &self,
        key: K,
        value: V,
    ) -> Result<Option<String>, StashError>
    where
        K: Into<String> + Send + Sync,
        V: Into<String> + Send + Sync,
    {
        let key = Into::<String>::into(key);
        Self::validate_key(&key)?;
        let previous: Option<String> = self.0
            .clone()
            .set(
                key,
                Into::<String>::into(value)
            ).await?;
        Ok(previous)
    }

    async fn delete<K>(
        &self,
        key: K,
    ) -> Result<Option<String>, StashError>
    where
        K: AsRef<str> + Send + Sync,
    {
        let previous: Option<String> = self.0
            .clone()
            .del(key.as_ref()).await?;
        Ok(previous)
    }
}

// TODO: Add unit tests for Redis stash using mock Redis server