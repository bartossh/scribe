mod dictionary;
mod repository;
mod settings;
mod trie;

use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use repository::interface::RepositoryProvider;
use repository::sql::{DatabaseStorage, Warehouse};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use web::{Data, Json};

/// VERSION shall be updated before creating release.
static VERSION: &str = "Scribe 1.0.0";

#[derive(Debug, Serialize, Deserialize)]
struct Version {
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogInput {
    log: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LogsOutput {
    logs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Query {
    prefix: Option<String>,
    words: Option<Vec<String>>,
    from: u64,
    to: u64,
}

struct ServerActor<T>
where
    T: RepositoryProvider + 'static,
{
    version: String,
    repo: Box<Arc<T>>,
    dict: Box<Arc<RwLock<dictionary::Module>>>,
}

impl<T> Clone for ServerActor<T>
where
    T: RepositoryProvider + 'static,
{
    fn clone(&self) -> Self {
        Self {
            version: self.version.clone(),
            repo: self.repo.clone(),
            dict: self.dict.clone(),
        }
    }
}

#[inline(always)]
#[get("/version")]
async fn version(state: Data<ServerActor<Warehouse>>) -> Result<impl Responder> {
    let v = Version {
        version: state.version.to_string(),
    };
    Ok(Json(v))
}

#[inline(always)]
#[post("/save")]
async fn save_log(
    input: Json<LogInput>,
    state: Data<ServerActor<Warehouse>>,
) -> Result<impl Responder> {
    let Ok(mut dict) = state.dict.write() else {
        return Err(error::ErrorInternalServerError(
            "Dictionary is not responding.",
        ));
    };
    let buf = dict.serialize(&input.log);
    if let Err(e) = state.repo.insert_log(&buf).await {
        return Err(error::ErrorInternalServerError(e.to_string()));
    };

    Ok(HttpResponse::Ok())
}

#[inline(always)]
#[post("/read")]
async fn read_logs(
    input: Json<Query>,
    state: Data<ServerActor<Warehouse>>,
) -> Result<impl Responder> {
    let from = Duration::from_nanos(input.from);
    let to = Duration::from_nanos(input.to);
    let Ok(mut logs) = state.repo.get_logs(&from, &to).await else {
        return Err(error::ErrorInternalServerError("Database not responding."));
    };

    let Ok(dict) = state.dict.read() else {
        return Err(error::ErrorInternalServerError(
            "Dictionary is not responding.",
        ));
    };

    if let Some(prefix) = input.prefix.as_ref() {
        logs = dict.filter_prefixed(prefix, logs);
    }

    if let Some(words) = input.words.as_ref() {
        logs = dict.filter_word(words, logs);
    }

    let mut output = LogsOutput { logs: Vec::new() };
    for log in logs.iter() {
        output.logs.push(dict.deserialize(&log));
    }

    Ok(Json(output))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let setup = match args.len() {
        0 | 1 => settings::Setup::default(),
        _ => settings::Setup::from_file(&args[1])?,
    };

    let Ok(mut repo) = Warehouse::new(DatabaseStorage::Ram).await else {
        return Err(std::io::Error::new::<String>(
            std::io::ErrorKind::NotConnected,
            "repository is not responding".to_string(),
        ));
    };

    if let Err(e) = repo.migrate().await {
        return Err(std::io::Error::new::<String>(
            std::io::ErrorKind::NotConnected,
            e.to_string(),
        ));
    };

    let repo = Box::new(Arc::new(repo));

    let service = ServerActor {
        version: VERSION.to_string(),
        repo: repo.clone(),
        dict: Box::new(Arc::new(RwLock::new(dictionary::Module::new(
            trie::Node::new(),
        )))),
    };

    println!("\nStarting scribe server at [ {} ]\n", setup.get_addr());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(version)
            .service(save_log)
            .service(read_logs)
    })
    .bind((setup.get_ip(), setup.get_port()))?
    .run()
    .await
    .unwrap_or_else(|e| println!("\nCannot run scribe server due to: {}\n", e));

    println!("\nStopping the scribe server.\n");

    repo.close().await;

    println!("All connections closed.\n");

    Ok(())
}
