use crate::dictionary::Module;
use std::io::Result;
use std::time::Duration;

/// RepositoryProvider provides full functionality of the persistent repository.
///
pub trait RepositoryProvider: Send + Sync + Clone {
    async fn migrate(&self) -> Result<()>;
    async fn insert_log(&self, input: &[u32]) -> Result<()>;
    async fn find_logs(&self, from: &Duration, to: &Duration) -> Result<Vec<Vec<u32>>>;
    async fn close(&self);
}

/// Stores Serializer in Self.
///
pub trait SerializerSaver {
    async fn save(&self, s: &Module) -> Result<()>;
}

/// Reads stored Serializer in Self.
///
pub trait SerializerReader {
    async fn read(&self) -> Result<Module>;
}
