use chrono::{DateTime, UTC};

use avro::codec::{ByteStream, AvroCodec};


use rusqlite;
use rusqlite::types::{ToSql, FromSql, ToSqlOutput};

use std::convert::From;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub struct Timestamp {
    pub timestamp: i64,
}

impl Timestamp {
    pub fn new(t: i64) -> Timestamp {
        Timestamp { timestamp: t }
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn now() -> Timestamp {
        Timestamp::from(UTC::now())
    }
}

impl From<DateTime<UTC>> for Timestamp {
    fn from(dt: DateTime<UTC>) -> Self {
        Timestamp::new(dt.timestamp())
    }
}

impl ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        self.timestamp.to_sql()
    }
}

impl AvroCodec for Timestamp {
    fn encode(&self) -> Vec<u8> {
        self.timestamp.encode()
    }

    fn decode(bytes: &mut ByteStream) -> Option<Self> {
        i64::decode(bytes).map(|ts| Timestamp::new(ts))
    }
}
