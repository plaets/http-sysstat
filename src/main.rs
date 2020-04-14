use actix_web::{
    Responder, HttpResponse, HttpServer, App, get,
    web::{Query, Data},
};
use systemstat::{System, Platform, data::CPULoad};
use serde::{Deserialize};
use std::io::Result;
use std::sync::Arc;

mod stats;
use stats::{StatsCollector, StatsConfig};

mod interval_collector;
use interval_collector::IntervalCollectorHandle;

mod cpu_load_collector;
use cpu_load_collector::create_cpu_collector;

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

struct AppData {
    cpu_load_collector: Arc<IntervalCollectorHandle<CPULoad>>,
}

#[get("/")]
async fn index(info: Query<IndexQuery>, data: Data<AppData>) -> impl Responder {
    let cfg = StatsConfig {
        date_format: info.date_format.unwrap_or(DateFormat::Epoch),
        human_readable: info.human_readable.unwrap_or(false),
        cpu_load_collector_handle: data.cpu_load_collector.clone(),
    };

    match info.output_format {
        Some(OutputFormat::Json) => get_json_stats(&cfg),
        Some(OutputFormat::Html) | None => HttpResponse::Ok().body("html"),
    }
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let collector_handle = Arc::new(create_cpu_collector());
    HttpServer::new(move || 
        App::new()
            .service(index)
            .data(AppData {
                cpu_load_collector: collector_handle.clone(),
            })
        )
        .bind("0.0.0.0:8080")?
        .run()
        .await
}


/* config:
 * scripts
 * cpu load average checking interval
 * cache
 * password protection
 * filter filesystems
 */
