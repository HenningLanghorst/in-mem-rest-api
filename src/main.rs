use serde_json::{json, Value};
use warp::http::Response;
use warp::path::FullPath;
use warp::Filter;

use crate::database::{ConcurrentDatabase, DatabaseAccess, DatabaseError};

mod database;

#[tokio::main]
async fn main() {
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

    warp::serve(post.or(get)).run(([127, 0, 0, 1], 3030)).await;
}
