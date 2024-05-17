use super::entities::{DictMongo, LogMongo};
use super::interface::RepositoryProvider;
use mongodb::bson::DateTime;
use mongodb::options::FindOptions;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, IndexModel,
};
use std::io::{Error, ErrorKind, Result};
use std::time::Duration;

const DATABASE_NAME: &str = "scribe";
const COLLECTION_LOGS: &str = "logs";

/// WarehouseMongo serves access to MongoDB repository via facade methods.
///
#[derive(Clone, Debug)]
pub struct WarehouseMongo {
    client: Client,
}

impl WarehouseMongo {
    /// New creates a new WarehouseMongo client.
    ///
    pub async fn new(connection_str: &str) -> Result<Self> {
        let Ok(mut client_options) = ClientOptions::parse_async(connection_str).await else {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("cannot connect to: {}", connection_str),
            ));
        };

        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();

        client_options.server_api = Some(server_api);

        let Ok(client) = Client::with_options(client_options) else {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("cannot connect to: {}", connection_str),
            ));
        };

        let Ok(_) = client
            .database("admin")
            .run_command(doc! { "ping": 1 }, None)
            .await
        else {
            return Err(Error::new(
                ErrorKind::ConnectionRefused,
                format!("cannot ping database on address: {}", connection_str),
            ));
        };

        Ok(Self { client })
    }
}

impl RepositoryProvider for WarehouseMongo {
    async fn migrate(&self) -> Result<()> {
        let index = IndexModel::builder().keys(doc! { "timestamp": 1 }).build();
        let db = self.client.database(DATABASE_NAME);

        match db
            .collection::<LogMongo>(COLLECTION_LOGS)
            .create_index(index, None)
            .await
        {
            Ok(_) => Ok(()),
            Err(_e) => Err(Error::new(ErrorKind::Other, "creating index failed")),
        }
    }

    async fn insert_log(&self, input: &[u32]) -> Result<()> {
        let db = self.client.database(DATABASE_NAME);

        let mut data = Vec::new();

        for elem in input {
            data.extend(elem.to_ne_bytes().to_vec());
        }
        let timestamp = DateTime::now();

        let Ok(_) = db
            .collection::<LogMongo>(COLLECTION_LOGS)
            .insert_one(
                LogMongo {
                    id: None,
                    data: data.to_vec(),
                    timestamp: timestamp,
                },
                None,
            )
            .await
        else {
            return Err(Error::new(
                ErrorKind::Other,
                format!("cannot insert log to collection : {}", COLLECTION_LOGS),
            ));
        };

        Ok(())
    }

    async fn find_logs(&self, from: &Duration, to: &Duration) -> Result<Vec<Vec<u32>>> {
        let db = self.client.database(DATABASE_NAME);
        let Ok(mut cursor) = db
            .collection::<LogMongo>(COLLECTION_LOGS)
            .find(
                doc! { "timestamp": doc! {
                    "$gte": DateTime::from_millis(from.as_millis() as i64), "$lte": DateTime::from_millis(to.as_millis() as i64)
                }
            },
                FindOptions::builder().sort(doc! {}).build(),
            )
            .await else {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("cannot get document field data form: {}", COLLECTION_LOGS),
                ));

            };
        let mut result = Vec::new();
        while let Ok(next_exists) = cursor.advance().await {
            if !next_exists {
                break;
            }
            let mut d: Vec<u32> = Vec::new();
            let Ok(log) = cursor.deserialize_current() else {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("cannot get document field data form: {}", COLLECTION_LOGS),
                ));
            };
            let data = log.data;
            for (i, _) in data.iter().enumerate().step_by(4) {
                d.push(u32::from_ne_bytes([
                    data[i],
                    data[i + 1],
                    data[i + 2],
                    data[i + 3],
                ]));
            }
            result.push(d);
        }

        Ok(result)
    }

    async fn close(&self) {
        self.client.clone().shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use super::super::interface::RepositoryProvider;
    use super::*;
    use std::time::Instant;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    const CONNECTION_STR_TEST: &str = "mongodb://scribe:scribe@localhost:27017/";

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
    async fn on_insert_should_insert_data_in_to_database() {
        let Ok(warehouse) = WarehouseMongo::new(CONNECTION_STR_TEST).await else {
            assert!(false);
            return;
        };
        let Ok(_) = warehouse.migrate().await else {
            assert!(false);
            return;
        };

        let data: Vec<u32> = get_data();

        let Ok(_) = warehouse.insert_log(&data).await else {
            assert!(false);
            return;
        };
        warehouse.close().await;
    }

    #[tokio::test]
    async fn on_insert_should_insert_data_in_to_database_and_read_the_data_with_proper_decoding() {
        let Ok(warehouse) = WarehouseMongo::new(CONNECTION_STR_TEST).await else {
            assert!(false);
            return;
        };
        let Ok(_) = warehouse.migrate().await else {
            assert!(false);
            return;
        };

        let data: Vec<u32> = get_data();

        let from = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let Ok(_) = warehouse.insert_log(&data).await else {
            assert!(false);
            return;
        };

        let to = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let Ok(result) = warehouse.find_logs(&from, &to).await else {
            assert!(false);
            return;
        };

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], data);
        warehouse.close().await;
    }

    #[tokio::test]
    async fn bench_insert_to_mongo() {
        let Ok(warehouse) = WarehouseMongo::new(CONNECTION_STR_TEST).await else {
            assert!(false);
            return;
        };
        let Ok(_) = warehouse.migrate().await else {
            assert!(false);
            return;
        };

        let data: Vec<u32> = get_data();

        let start = Instant::now();

        for _ in 0..BENCH_LOOP {
            let Ok(_) = warehouse.insert_log(&data).await else {
                assert!(false);
                return;
            };
        }

        let duration = start.elapsed();

        println!(
            "Time elapsed in bench_insert_to_mongo is: {:?}",
            duration / BENCH_LOOP as u32
        );
        warehouse.close().await;
    }

    #[tokio::test]
    async fn bench_find_in_mongo() {
        let Ok(warehouse) = WarehouseMongo::new(CONNECTION_STR_TEST).await else {
            assert!(false);
            return;
        };
        let Ok(_) = warehouse.migrate().await else {
            assert!(false);
            return;
        };

        let data: Vec<u32> = get_data();

        for _ in 0..INSERTS * 2 {
            let Ok(_) = warehouse.insert_log(&data).await else {
                assert!(false);
                return;
            };
        }

        std::thread::sleep(Duration::from_millis(10));

        let from = SystemTime::now()
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

        let to = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let start = Instant::now();

        let Ok(result) = warehouse.find_logs(&from, &to).await else {
            println!("Cannot get logs from warehouse.");
            assert!(false);
            return;
        };

        let duration = start.elapsed();

        assert_eq!(result.len(), num_inserted_on_time);
        assert_eq!(data, result[0]);

        warehouse.close().await;

        println!(
            "Time elapsed in bench_find_in_mongo is: {:?}",
            duration / BENCH_LOOP as u32
        );
    }
}
