mod commands;
mod entities;
pub mod interface;
pub mod mongo;
pub mod sql;
use crate::settings::Setup;
use std::{io::Result, time::Duration};

#[derive(Clone, Debug)]
pub enum Repository {
    Mongo(mongo::WarehouseMongo),
    Sql(sql::WarehouseSql),
}

impl Repository {
    pub async fn new(s: &Setup) -> Result<Self> {
        let conn_str = s.get_connection_str();

        if conn_str.contains("mongodb") {
            let m = mongo::WarehouseMongo::new(&conn_str).await?;
            return Ok(Self::Mongo(m));
        }
        if !conn_str.is_empty() {
            let s = sql::WarehouseSql::new(sql::DatabaseStorage::Path(conn_str)).await?;
            return Ok(Self::Sql(s));
        }
        let s = sql::WarehouseSql::new(sql::DatabaseStorage::Ram).await?;
        Ok(Self::Sql(s))
    }
}

impl interface::RepositoryProvider for Repository {
    async fn migrate(&self) -> Result<()> {
        match &self {
            Repository::Mongo(r) => r.migrate().await,
            Repository::Sql(r) => r.migrate().await,
        }
    }
    async fn insert_log(&self, input: &[u32]) -> Result<()> {
        match &self {
            Repository::Mongo(r) => r.insert_log(input).await,
            Repository::Sql(r) => r.insert_log(input).await,
        }
    }

    async fn find_logs(&self, from: &Duration, to: &Duration) -> Result<Vec<Vec<u32>>> {
        match &self {
            Repository::Mongo(r) => r.find_logs(from, to).await,
            Repository::Sql(r) => r.find_logs(from, to).await,
        }
    }

    async fn close(&self) {
        match &self {
            Repository::Mongo(r) => r.close().await,
            Repository::Sql(r) => r.close().await,
        }
    }
}
