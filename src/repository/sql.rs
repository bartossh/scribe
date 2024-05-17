use super::commands::SQL_COMMANDS;
use super::entities::{DictSql, LogSql};
use super::interface::RepositoryProvider;
use super::interface::{SerializerReader, SerializerSaver};
use crate::dictionary::Module;
use crate::trie::Node;
use sqlx::{sqlite::SqlitePool, FromRow};
use std::io::{Error, ErrorKind, Result};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub enum DatabaseStorage {
    Ram,
    Path(String),
}

/// WarehouseSql serves access to MongoDB repository via facade methods.
///
#[derive(Debug, Clone)]
pub struct WarehouseSql {
    pool: SqlitePool,
}

impl WarehouseSql {
    /// Cerate a new Warehouse connected to SQLite database.
    ///
    pub async fn new(dbs: DatabaseStorage) -> Result<Self> {
        let url = match dbs {
            DatabaseStorage::Ram => "sqlite::memory:".to_string(),
            DatabaseStorage::Path(s) => s,
        };
        let Ok(pool) = SqlitePool::connect(&url).await else {
            return Err(Error::new(ErrorKind::NotConnected, "connection error"));
        };
        Ok(Self { pool })
    }
}

impl RepositoryProvider for WarehouseSql {
    async fn migrate(&self) -> Result<()> {
        let Ok(mut conn) = self.pool.acquire().await else {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                "cannot acquire connection",
            ));
        };
        for migration in SQL_COMMANDS {
            let Ok(_) = sqlx::query(&migration).execute(&mut *conn).await else {
                return Err(Error::new(ErrorKind::NotConnected, "cannot acquire pool"));
            };
        }
        Ok(())
    }

    /// Insert single log data to Warehouse SQLite database.
    ///
    async fn insert_log(&self, input: &[u32]) -> Result<()> {
        let Ok(mut conn) = self.pool.acquire().await else {
            return Err(Error::new(ErrorKind::NotConnected, "cannot acquire pool"));
        };
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let mut data = Vec::new();

        for elem in input {
            data.extend(elem.to_ne_bytes().to_vec());
        }

        let Ok(_) = sqlx::query("INSERT INTO logs (timestamp, data) VALUES (?1, ?2)")
            .bind(timestamp)
            .bind(data)
            .execute(&mut *conn)
            .await
        else {
            return Err(Error::new(ErrorKind::Interrupted, "cannot execute query"));
        };

        Ok(())
    }

    /// Gets data in time span.
    ///  
    async fn find_logs(&self, from: &Duration, to: &Duration) -> Result<Vec<Vec<u32>>> {
        let Ok(mut conn) = self.pool.acquire().await else {
            return Err(Error::new(ErrorKind::NotConnected, "cannot acquire pool"));
        };
        let Ok(rows) = sqlx::query("SELECT * FROM logs WHERE timestamp BETWEEN ? AND ?")
            .bind(from.as_nanos() as i64)
            .bind(to.as_nanos() as i64)
            .fetch_all(&mut *conn)
            .await
        else {
            return Err(Error::new(ErrorKind::Interrupted, "cannot execute query"));
        };

        let mut data = Vec::new();
        for rec in rows {
            let mut d: Vec<u32> = Vec::new();
            let Ok(log) = LogSql::from_row(&rec) else {
                return Err(Error::new(ErrorKind::Interrupted, "cannot execute query"));
            };
            for (i, _) in log.data.iter().enumerate().step_by(4) {
                d.push(u32::from_ne_bytes([
                    log.data[i],
                    log.data[i + 1],
                    log.data[i + 2],
                    log.data[i + 3],
                ]));
            }
            data.push(d);
        }

        Ok(data)
    }

    async fn close(&self) {
        self.pool.close().await;
    }
}

impl SerializerReader for WarehouseSql {
    #[inline]
    async fn read(&self) -> Result<Module> {
        let Ok(mut conn) = self.pool.acquire().await else {
            return Err(Error::new(ErrorKind::NotConnected, "cannot acquire pool"));
        };

        let Ok(mut rows) = sqlx::query("SELECT * FROM serializer")
            .fetch_all(&mut *conn)
            .await
        else {
            return Err(Error::new(ErrorKind::Interrupted, "cannot execute query"));
        };

        let mut m: HashMap<String, u32> = HashMap::new();

        for rec in rows {
            let Ok(dict) = DictSql::from_row(&rec) else {
                return Err(Error::new(ErrorKind::Interrupted, "cannot execute query"));
            };
            m.insert(dict.word, dict.num as u32);
        }

        let graph = Node::new();

        let mut s = Module::new(graph);
        s.set_map_from(m);

        Ok(s)
    }
}

impl SerializerSaver for WarehouseSql {
    #[inline]
    async fn save(&self, s: &Module) -> Result<()> {
        let Ok(mut transaction) = self.pool.begin().await else {
            return Err(Error::new(
                ErrorKind::NotConnected,
                "cannot begin transaction pool",
            ));
        };

        for (w, n) in s.iter() {
            let Ok(_) = sqlx::query("INSERT INTO serializer (word, num) VALUES (?1, ?2)")
                .bind(w)
                .bind(*n as i32)
                .execute(&mut *transaction)
                .await
            else {
                return Err(Error::new(
                    ErrorKind::Interrupted,
                    "cannot execute transaction",
                ));
            };
        }

        let Ok(_) = transaction.commit().await else {
            return Err(Error::new(ErrorKind::Interrupted, "cannot execute query"));
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::interface::{SerializerReader, SerializerSaver};
    use super::*;
    use crate::dictionary::Module;
    use std::time::Instant;

    const BENCH_LOOP: usize = 1000;
    const INSERTS: usize = 100;

    fn get_data() -> Vec<u32> {
        vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4,
            5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
            12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3,
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7,
            8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
            11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
            14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1,
            2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5,
            6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
            13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4,
            5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
            12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3,
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7,
            8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
            11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
            14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1,
            2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5,
            6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
            13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4,
            5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
            12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
            18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3,
            4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7,
            8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
            11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13,
            14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
            17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1,
            2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5,
            6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
            13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4,
            5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8,
            9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
            12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21,
        ]
    }

    #[tokio::test]
    async fn on_insert_should_insert_data_in_to_database_and_read_the_data_without_side_effects() {
        let data: Vec<u32> = get_data();

        let Ok(mut warehouse) = WarehouseSql::new(DatabaseStorage::Ram).await else {
            println!("Cannot create warehouse");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.migrate().await else {
            println!("Cannot migrate warehouse.");
            assert!(false);
            return;
        };

        for _ in 0..INSERTS * 2 {
            let Ok(()) = warehouse.insert_log(&data).await else {
                println!("Cannot insert logs into warehouse.");
                assert!(false);
                return;
            };
        }

        let time_0 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let num_inserted_on_time = INSERTS;
        for _ in 0..num_inserted_on_time {
            let Ok(()) = warehouse.insert_log(&data).await else {
                println!("Cannot insert logs into warehouse.");
                assert!(false);
                return;
            };
        }

        let time_1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let Ok(result) = warehouse.find_logs(&time_0, &time_1).await else {
            println!("Cannot get logs from warehouse.");
            assert!(false);
            return;
        };

        assert_eq!(result.len(), num_inserted_on_time);
        assert_eq!(data, result[0]);

        warehouse.close().await;
    }

    #[tokio::test]
    async fn bench_insert_sql_ram() {
        let data: Vec<u32> = get_data();
        let Ok(warehouse) = WarehouseSql::new(DatabaseStorage::Ram).await else {
            println!("Cannot create warehouse");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.migrate().await else {
            println!("Cannot migrate warehouse.");
            assert!(false);
            return;
        };

        let start = Instant::now();

        for _ in 0..BENCH_LOOP {
            let Ok(()) = warehouse.insert_log(&data).await else {
                println!("Cannot insert logs into warehouse.");
                assert!(false);
                return;
            };
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in bench_insert_sql_ram is: {:?}",
            duration / BENCH_LOOP as u32
        );

        warehouse.close().await;
    }

    #[tokio::test]
    async fn bench_find_sql_ram() {
        let data: Vec<u32> = get_data();
        let Ok(warehouse) = WarehouseSql::new(DatabaseStorage::Ram).await else {
            println!("Cannot create warehouse");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.migrate().await else {
            println!("Cannot migrate warehouse.");
            assert!(false);
            return;
        };

        let time_0 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        for _ in 0..INSERTS {
            let Ok(()) = warehouse.insert_log(&data).await else {
                println!("Cannot insert logs into warehouse.");
                assert!(false);
                return;
            };
        }

        let time_1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let start = Instant::now();

        for _ in 0..BENCH_LOOP {
            let Ok(_) = warehouse.find_logs(&time_0, &time_1).await else {
                println!("Cannot get logs from warehouse.");
                assert!(false);
                return;
            };
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in bench_find_sql_ram is: {:?}",
            duration / BENCH_LOOP as u32
        );

        warehouse.close().await;
    }

    #[tokio::test]
    async fn test_serializer_save() {
        let mut hm = HashMap::new();

        for (i, w) in ["a", "b", "c", "d"].iter().enumerate() {
            hm.insert(w.to_string(), i as u32);
        }
        let graph = Node::new();

        let mut s: Module = Module::new(graph);
        s.set_map_from(hm);

        let Ok(mut warehouse) = WarehouseSql::new(DatabaseStorage::Ram).await else {
            println!("Cannot create warehouse");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.migrate().await else {
            println!("Cannot migrate warehouse.");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.save(&s).await else {
            println!("Cannot save serializer warehouse");
            assert!(false);
            return;
        };

        warehouse.close().await;
    }

    #[tokio::test]
    async fn test_serializer_read() {
        let mut hm = HashMap::new();

        for (i, w) in ["a", "b", "c", "d"].iter().enumerate() {
            hm.insert(w.to_string(), i as u32);
        }

        let graph = Node::new();

        let mut s: Module = Module::new(graph);
        s.set_map_from(hm);

        let Ok(mut warehouse) = WarehouseSql::new(DatabaseStorage::Ram).await else {
            println!("Cannot create warehouse");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.migrate().await else {
            println!("Cannot migrate warehouse.");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.save(&s).await else {
            println!("Cannot save serializer warehouse");
            assert!(false);
            return;
        };

        let Ok(_) = warehouse.read().await else {
            println!("Cannot save serializer warehouse");
            assert!(false);
            return;
        };

        warehouse.close().await;
    }
}
