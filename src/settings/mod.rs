use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

/// Setup contains scribe setup parameters.
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setup {
    ip: IpAddr,
    port: u16,
    mongo_db: String,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 8000,
            mongo_db: "mongodb://scribe:scribe@localhost:27017/".to_string(),
        }
    }
}

impl Setup {
    /// Deserializes Setup from file under given path.
    ///
    pub fn from_file(path: &str) -> std::io::Result<Setup> {
        let f = std::fs::File::open(path)?;
        let Ok(s) = serde_yaml::from_reader(f) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot read the file".to_string(),
            ));
        };

        Ok(s)
    }

    /// Returns address in form of ip and port like: `0.0.0.0:8000`.
    ///
    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.ip.to_string(), self.port.to_string())
    }

    pub fn get_ip(&self) -> String {
        self.ip.to_string()
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_mongo_connection_str(&self) -> String {
        self.mongo_db.clone()
    }
}
