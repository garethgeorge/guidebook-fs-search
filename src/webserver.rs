use std::io::Cursor;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::index::{Document, Index};
use anyhow::Result;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use rocket::config::Config as RocketConfig;
use rocket::fs::NamedFile;
use rocket::http::ContentType;
use rocket::response::content::RawText;
use rocket::response::Responder;
use rocket::serde::json::{json, Value};
use rocket::{get, route, routes, Response};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

pub static WEB_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/web/dist");

#[derive(Clone)]
struct WebServerState {
    db: Arc<dyn Index>,
}
lazy_static! {
    static ref state: Mutex<Option<WebServerState>> = Mutex::new(None);
}

pub fn set_state(database: Arc<dyn Index>) -> Result<()> {
    state.lock().unwrap().replace(WebServerState {
        db: database.clone(),
    });
    return Ok(());
}

fn get_state() -> WebServerState {
    let guard = state.lock().expect("failed to lock state");
    return guard.as_ref().unwrap().clone();
}

pub fn serve() -> Result<()> {
    let mut config = RocketConfig::default();
    config.address = IpAddr::from([0, 0, 0, 0]);
    config.port = 8080;

    let rocket_future = rocket::custom(config)
        .mount("/", routes![route_query, route_index])
        .launch();

    let rt = Runtime::new().unwrap();
    rt.block_on(rocket_future);

    return Ok(());
}

#[get("/<fpath..>")]
fn route_index<'r>(fpath: PathBuf) -> Option<(ContentType, String)> {
    let mut fpath = fpath;
    if fpath.ends_with("/") {
        fpath.push("index.html");
    }

    let content = WEB_DIR.get_file(fpath.to_str().unwrap_or("/"));
    if content.is_none() {
        return None;
    }

    let ext = fpath
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();
    let content_type = ContentType::from_extension(ext).unwrap_or(ContentType::Text);
    let content = content.unwrap();

    return Some((
        content_type,
        content.contents_utf8().unwrap_or("").to_string(),
    ));
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QueryResult {
    results: Vec<Document>,
    error: Option<String>,
    latency: u64,
}

impl QueryResult {
    fn new(results: Vec<Document>, latency: u128) -> QueryResult {
        QueryResult {
            results: results,
            error: None,
            latency: latency as u64,
        }
    }

    fn new_err(error: String) -> QueryResult {
        QueryResult {
            results: Vec::new(),
            error: Some(error),
            latency: 0,
        }
    }
}

#[get("/query?<query>&<offset>&<limit>")]
fn route_query(query: Option<String>, offset: Option<String>, limit: Option<String>) -> Value {
    if query.is_none() {
        return json!(QueryResult::new_err(
            "query parameter is required".to_string()
        ));
    }
    let query = query.unwrap();

    let offset: u64 = offset.unwrap_or("0".to_string()).parse().unwrap_or(0);
    let limit: u64 = limit.unwrap_or("25".to_string()).parse().unwrap_or(0);

    if limit > 500 {
        return json!(QueryResult::new_err("limit must be <= 100".to_string()));
    }

    let s = get_state();

    let now = std::time::SystemTime::now();
    let res = s.db.search(&query, limit as usize, offset as usize);
    let took = now.elapsed().unwrap().as_millis();

    match res {
        Ok(results) => {
            return json!(QueryResult::new(results, took));
        }
        Err(e) => {
            return json!(QueryResult::new_err(e.to_string()));
        }
    }
}
