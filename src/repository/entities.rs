use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Dict contains dictionary mapping.
///
#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct Dict {
    pub word: String,
    pub num: i32,
}

/// Log contains log data in binary format.
/// Use Dict to decode binary format via dictionary mapping.
///
#[derive(FromRow, Debug, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub timestamp: i64,
    pub data: Vec<u8>,
}
