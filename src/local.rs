use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use crate::{Stash, StashError};

/// A local in-memory [`Stash`].
/// 
/// This is essentially just a wrapper around [`HashMap`] that implements
/// [`Stash`] and can be shared across threads.
#[derive(Clone)]
pub struct LocalStash(Arc<Mutex<HashMap<String, String>>>);

impl Default for LocalStash {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalStash {
    /// Creates a new [`LocalStash`].
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Returns `true` if the [`LocalStash`] is empty, otherwise returns
    /// `false`.
    #[inline(always)]
    #[must_use]
    pub async fn is_empty(&self) -> bool {
        self.0.lock().await.is_empty()
    }

    /// Returns the number of stashed keys in the [`LocalStash`].
    #[inline(always)]
    #[must_use]
    pub async fn len(&self) -> usize {
        self.0.lock().await.len()
    }
}

#[async_trait::async_trait]
impl Stash for LocalStash {
    async fn fetch<K>(
        &self,
        key: K,
    ) -> Result<Option<String>, StashError>
    where
        K: AsRef<str> + Send + Sync,
    {
        Ok(
            self.0
                .lock()
                .await
                .get(key.as_ref())
                .map(|value| value.to_owned())
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
        Ok(
            self.0
                .lock()
                .await
                .insert(key, value.into())
        )
    }

    async fn delete<K>(
        &self,
        key: K,
    ) -> Result<Option<String>, StashError>
    where
        K: AsRef<str> + Send + Sync,
    {
        Ok(
            self.0
                .lock()
                .await
                .remove(key.as_ref())
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{LocalStash, Stash};

    #[tokio::test]
    async fn create_local_stash() {
        let stash = LocalStash::new();
        assert!(stash.is_empty().await);
        assert_eq!(stash.len().await, 0);
    }

    #[tokio::test]
    async fn stash_locally() {
        let stash = LocalStash::new();
        assert_eq!(stash.stash("test", "value123").await.unwrap(), None);
        assert_eq!(stash.stash("user:1:name", "Alice").await.unwrap(), None);
        assert_eq!(stash.stash("user:2:name", "Bob").await.unwrap(), None);
        assert_eq!(stash.stash("user:3:name", "Charlie").await.unwrap(), None);
        assert_eq!(stash.len().await, 4);
        assert_eq!(stash.fetch("user:1:name").await.unwrap(), Some("Alice".to_owned()));
        assert_eq!(stash.fetch("user:2:name").await.unwrap(), Some("Bob".to_owned()));
        assert_eq!(stash.fetch("user:3:name").await.unwrap(), Some("Charlie".to_owned()));
    }

    #[tokio::test]
    async fn invalid_stash_key() {
        let stash = LocalStash::new();
        assert!(stash.stash("invalid key", "value").await.is_err());
    }

    #[tokio::test]
    async fn override_key() {
        let stash = LocalStash::new();
        assert_eq!(stash.stash("test", "value123").await.unwrap(), None);
        assert_eq!(stash.stash("test", "value1234").await.unwrap(), Some("value123".to_owned()));
        assert_eq!(stash.fetch("test").await.unwrap(), Some("value1234".to_owned()));
    }

    #[tokio::test]
    async fn delete_key() {
        let stash = LocalStash::new();
        assert_eq!(stash.stash("key1", "1").await.unwrap(), None);
        assert_eq!(stash.stash("key2", "2").await.unwrap(), None);
        assert_eq!(stash.stash("key3", "3").await.unwrap(), None);
        assert_eq!(stash.len().await, 3);

        assert_eq!(stash.delete("key3").await.unwrap(), Some("3".to_owned()));
        assert_eq!(stash.delete("key1").await.unwrap(), Some("1".to_owned()));
        assert_eq!(stash.delete("key2").await.unwrap(), Some("2".to_owned()));
    }
}