use actix_web::{
    Responder, HttpResponse, HttpServer, App, get,
    web::{Query},
};
use serde::{Deserialize};
use std::io::Result;

mod stats;
use stats::{StatsCollector, StatsConfig};

fn get_json_stats(cfg: &StatsConfig) -> HttpResponse {
    let collector = StatsCollector::new(cfg);
    HttpResponse::Ok().json(collector.get_stats())
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DateFormat {
    Epoch,
    Local,
    Utc,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum OutputFormat {
    Json,
    Html,
}

#[derive(Deserialize)]
struct IndexQuery {
    output_format: Option<OutputFormat>,
    date_format: Option<DateFormat>,
    human_readable: Option<bool>
}

#[get("/")]
async fn index(info: Query<IndexQuery>) -> impl Responder {
    let cfg = StatsConfig {
        date_format: info.date_format.unwrap_or(DateFormat::Epoch),
        human_readable: info.human_readable.unwrap_or(false),
    };

    match info.output_format {
        Some(OutputFormat::Json) => get_json_stats(&cfg),
        Some(OutputFormat::Html) | None => HttpResponse::Ok().body("html"),
    }
}

//looks like actix has a memory leak somewhere 
//this sucks
#[actix_rt::main]
async fn main() -> Result<()> {
    HttpServer::new(|| App::new().service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
