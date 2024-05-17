use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use std::io::{Error, ErrorKind, Result};

/// WarehouseMongo serves access to MongoDB repository via facade methods.
///
pub struct WarehouseMongo {
    client: Client,
}

impl WarehouseMongo {
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
