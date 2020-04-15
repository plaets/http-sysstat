use actix_web::{
    Responder, HttpResponse, HttpRequest, HttpServer, App, get,
    web::{Query, Data},
};
use serde::{Deserialize};
use serde_json::{Value};
use serde_urlencoded;
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

mod stats;
use stats::get_all;

use http_sysstat::plugin_lib::stats_collector::{DateFormat, StatsCollector, StatsConfig};

fn get_json_stats(cfg: &StatsConfig, data: &AppData) -> HttpResponse {
    let collectors = data.collectors.clone();
    let collectors_guard = collectors.lock().unwrap();
    HttpResponse::Ok().json(
        (*collectors_guard).iter()
        .map(|c| ((*c).name(), (*c).collect(&cfg).unwrap()))
        .collect::<HashMap<&'static str, Value>>()
    )
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
    collectors: Arc<Mutex<Vec<Box<dyn StatsCollector>>>>
}

#[get("/")]
async fn index(info: Query<IndexQuery>, data: Data<AppData>, req: HttpRequest) -> impl Responder {
    let cfg = StatsConfig {
        date_format: info.date_format.unwrap_or(DateFormat::Epoch),
        human_readable: info.human_readable.unwrap_or(false),
        query_other: serde_urlencoded::from_str(req.query_string()).unwrap(),
    };

    match info.output_format {
        Some(OutputFormat::Json) => get_json_stats(&cfg, &data),
        Some(OutputFormat::Html) | None => HttpResponse::Ok().body("html"),
    }
}

#[actix_rt::main]
async fn main() -> Result<()> {
    let collectors = Arc::new(Mutex::new(get_all()));

    HttpServer::new(move || 
        App::new()
            .service(index)
            .data(AppData {
                collectors: collectors.clone(),
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
