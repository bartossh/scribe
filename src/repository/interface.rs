use std::io::Result;
use std::time::Duration;

/// RepositoryProvider provides full functionality of the persistent repository.
///
pub trait RepositoryProvider: Send + Sync + Clone {
    async fn migrate(&mut self) -> Result<()>;
    async fn insert_log(&self, input: &[u32]) -> Result<()>;
    async fn get_logs(&self, from: &Duration, to: &Duration) -> Result<Vec<Vec<u32>>>;
    async fn close(&self);
}
