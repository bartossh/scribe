mod dictionary;
mod repository;
mod tries;

use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use serde::{Deserialize, Serialize};
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
struct Input {
    log: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    logs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Query {
    from: u64,
    to: u64,
}

#[derive(Debug, Clone)]
struct ServerActor {
    version: String,
    repo: Box<Arc<repository::Warehouse>>,
    dict: Box<Arc<RwLock<dictionary::Module>>>,
}

#[inline(always)]
#[get("/version")]
async fn version(state: Data<ServerActor>) -> Result<impl Responder> {
    let v = Version {
        version: state.version.to_string(),
    };
    Ok(Json(v))
}

#[inline(always)]
#[post("/save")]
async fn save_log(input: Json<Input>, state: Data<ServerActor>) -> Result<impl Responder> {
    let Ok(mut dict) = state.dict.write() else {
        return Err(error::ErrorInternalServerError(
            "Dictionary is not responding.",
        ));
    };
    let buf = dict.serialize(&input.log);
    let Ok(()) = state.repo.insert_log(&buf).await else {
        return Err(error::ErrorInternalServerError("Database not responding."));
    };

    Ok(HttpResponse::Ok())
}

#[inline(always)]
#[post("/read")]
async fn read_log(input: Json<Query>, state: Data<ServerActor>) -> Result<impl Responder> {
    let from = Duration::from_nanos(input.from);
    let to = Duration::from_nanos(input.to);
    let Ok(logs) = state.repo.get_logs(&from, &to).await else {
        return Err(error::ErrorInternalServerError("Database not responding."));
    };
    let Ok(dict) = state.dict.read() else {
        return Err(error::ErrorInternalServerError(
            "Dictionary is not responding.",
        ));
    };
    let mut output = Output { logs: Vec::new() };
    for log in logs.iter() {
        output.logs.push(dict.deserialize(&log));
    }

    Ok(Json(output))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let Ok(mut repo) = repository::Warehouse::new(repository::DatabaseStorage::Ram).await else {
        return Err(std::io::Error::new::<String>(
            std::io::ErrorKind::NotConnected,
            "repository is not responding".to_string(),
        ));
    };

    let Ok(()) = repo.migrate().await else {
        return Err(std::io::Error::new::<String>(
            std::io::ErrorKind::NotConnected,
            "repository is not responding".to_string(),
        ));
    };

    let repo = Box::new(Arc::new(repo));

    let service = ServerActor {
        version: VERSION.to_string(),
        repo: repo.clone(),
        dict: Box::new(Arc::new(RwLock::new(dictionary::Module::new()))),
    };

    println!("\nStarting scribe server at [ 127.0.0.1:8080 ]\n");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(version)
            .service(save_log)
            .service(read_log)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .unwrap_or_else(|e| println!("\nCannot run scribe server due to: {}\n", e));

    println!("\nStopping the scribe server.\n");
    repo.close().await;

    println!("All connections closed.\n");

    Ok(())
}
