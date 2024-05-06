mod migrations;
use crate::dictionary::{Module, SerializerReader, SerializerSaver};
use sqlx::{sqlite::SqlitePool, Error, FromRow};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(FromRow, Debug)]
struct Dict {
    word: String,
    num: i32,
}

#[derive(FromRow, Debug)]
struct Log {
    id: i64,
    timestamp: i64,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum DatabaseStorage {
    Ram,
    Path(String),
}

#[derive(Debug, Clone)]
pub struct Warehouse {
    pool: SqlitePool,
}

impl Warehouse {
    /// Cerate a new Warehouse connected to SQLite database.
    ///
    pub async fn new(dbs: DatabaseStorage) -> Result<Self, Error> {
        let url = match dbs {
            DatabaseStorage::Ram => "sqlite::memory:".to_string(),
            DatabaseStorage::Path(s) => s,
        };
        let pool = SqlitePool::connect(&url).await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&mut self) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await?;
        for migration in migrations::COMMANDS {
            sqlx::query(&migration).execute(&mut *conn).await?;
        }
        Ok(())
    }

    /// Insert single log data to Warehouse SQLite database.
    ///
    pub async fn insert_log(&self, input: &[u32]) -> Result<(), Error> {
        let mut conn = self.pool.acquire().await?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let mut data = Vec::new();

        for elem in input {
            data.extend(elem.to_ne_bytes().to_vec());
        }

        sqlx::query("INSERT INTO logs (timestamp, data) VALUES (?1, ?2)")
            .bind(timestamp)
            .bind(data)
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    /// Gets data in time span.
    ///  
    pub async fn get_logs(&self, from: &Duration, to: &Duration) -> Result<Vec<Vec<u32>>, Error> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query("SELECT * FROM logs WHERE timestamp BETWEEN ? AND ?")
            .bind(from.as_nanos() as i64)
            .bind(to.as_nanos() as i64)
            .fetch_all(&mut *conn)
            .await?;

        let mut data = Vec::new();
        for rec in rows {
            let mut d: Vec<u32> = Vec::new();
            let log = Log::from_row(&rec)?;
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

    pub async fn close(&self) {
        self.pool.close().await;
    }
}

impl SerializerReader for Warehouse {
    #[inline]
    async fn read(&self) -> Result<Module, Error> {
        let mut conn = self.pool.acquire().await?;

        let rows = sqlx::query("SELECT * FROM serializer")
            .fetch_all(&mut *conn)
            .await?;

        let mut m: HashMap<String, u32> = HashMap::new();

        for rec in rows {
            let dict: Dict = Dict::from_row(&rec)?;
            m.insert(dict.word, dict.num as u32);
        }

        let mut s = Module::new();
        s.set_map_from(m);

        Ok(s)
    }
}

impl SerializerSaver for Warehouse {
    #[inline]
    async fn save(&self, s: &Module) -> Result<(), Error> {
        let mut transaction = self.pool.begin().await?;

        for (w, n) in s.iter() {
            sqlx::query("INSERT INTO serializer (word, num) VALUES (?1, ?2)")
                .bind(w)
                .bind(*n as i32)
                .execute(&mut *transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::{Module, SerializerReader, SerializerSaver};
    use std::time::Instant;

    const bench_loops: usize = 1000;

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
    async fn test_insert_and_get() {
        let data: Vec<u32> = get_data();

        let Ok(mut warehouse) = Warehouse::new(DatabaseStorage::Ram).await else {
            println!("Cannot create warehouse");
            assert!(false);
            return;
        };

        let Ok(()) = warehouse.migrate().await else {
            println!("Cannot migrate warehouse.");
            assert!(false);
            return;
        };

        for _ in 0..10 {
            let Ok(()) = warehouse.insert_log(&data).await else {
                println!("Cannot insert logs into warehouse.");
                assert!(false);
                return;
            };
        }

        let time_0 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let num_inserted_on_time = 25;
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

        let Ok(result) = warehouse.get_logs(&time_0, &time_1).await else {
            println!("Cannot get logs from warehouse.");
            assert!(false);
            return;
        };

        assert_eq!(data, result[0]);
        assert_eq!(result.len(), num_inserted_on_time);

        warehouse.close().await;
    }

    #[tokio::test]
    async fn test_insert_bench() {
        let data: Vec<u32> = get_data();
        let Ok(mut warehouse) = Warehouse::new(DatabaseStorage::Ram).await else {
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

        for _ in 0..bench_loops {
            let Ok(()) = warehouse.insert_log(&data).await else {
                println!("Cannot insert logs into warehouse.");
                assert!(false);
                return;
            };
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_insert_bench is: {:?}",
            duration / bench_loops as u32
        );

        warehouse.close().await;
    }

    #[tokio::test]
    async fn test_get_bench() {
        let data: Vec<u32> = get_data();
        let Ok(mut warehouse) = Warehouse::new(DatabaseStorage::Ram).await else {
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

        for _ in 0..100 {
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

        for _ in 0..bench_loops {
            let Ok(_) = warehouse.get_logs(&time_0, &time_1).await else {
                println!("Cannot get logs from warehouse.");
                assert!(false);
                return;
            };
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in test_get_bench is: {:?}",
            duration / bench_loops as u32
        );

        warehouse.close().await;
    }

    #[tokio::test]
    async fn test_serializer_save() {
        let mut hm = HashMap::new();

        for (i, w) in ["a", "b", "c", "d"].iter().enumerate() {
            hm.insert(w.to_string(), i as u32);
        }

        let mut s = Module::new();
        s.set_map_from(hm);

        let Ok(mut warehouse) = Warehouse::new(DatabaseStorage::Ram).await else {
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

        let mut s = Module::new();
        s.set_map_from(hm);

        let Ok(mut warehouse) = Warehouse::new(DatabaseStorage::Ram).await else {
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
