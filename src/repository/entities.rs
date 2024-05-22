use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Dict contains dictionary mapping.
///
#[derive(FromRow, Debug)]
pub struct DictSql {
    pub word: String,
    pub num: i32,
}

/// Dict contains dictionary mapping.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct DictMongo {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub word: String,
    pub num: i32,
}

/// Log contains log data in binary format.
/// Use Dict to decode binary format via dictionary mapping.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct LogMongo {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub timestamp: DateTime,
    pub data: Vec<u8>,
}

/// Log contains log data in binary format.
/// Use Dict to decode binary format via dictionary mapping.
///
#[derive(FromRow, Debug)]
pub struct LogSql {
    pub id: i64,
    pub timestamp: i64,
    pub data: Vec<u8>,
}
