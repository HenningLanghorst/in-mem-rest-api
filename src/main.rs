use std::error::Error;
use std::net::SocketAddr;
use serde_json::{json, Value};
use warp::http::Response;
use warp::path::FullPath;
use warp::Filter;
use clap::Parser;
use crate::cli_parameters::CliParams;

use crate::database::{ConcurrentDatabase, DatabaseAccess, DatabaseError};

mod database;
mod cli_parameters;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let params: CliParams = CliParams::parse();
    let socket_address: SocketAddr = params.socket_address.parse::<SocketAddr>()
        .map_err(|e| format!("Cannot parse socket address: {}", e))?;

    let original: ConcurrentDatabase = DatabaseAccess::new();

    let database: ConcurrentDatabase = original.clone();
    let get = warp::get()
        .and(warp::path::full())
        .map(move |path: FullPath| match database.get(path.as_str()) {
            Ok(value) => Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(value.to_string()),
            Err(DatabaseError { message }) => Response::builder()
                .status(500)
                .body(json!({ "error": message }).to_string()),
        });

    let database = original.clone();
    let post = warp::post()
        .and(warp::path::full())
        .and(warp::body::json())
        .map(move |path: FullPath, json: Value| {
            match database.clone().insert(path.as_str(), json) {
                Ok(value) => Response::builder()
                    .status(201)
                    .header("Content-Type", "application/json")
                    .body((*value).to_string()),
                Err(DatabaseError { message }) => Response::builder()
                    .status(500)
                    .body(json!({ "error": message }).to_string()),
            }
        });

    warp::serve(post.or(get)).run(socket_address).await;

    Ok(())
}
