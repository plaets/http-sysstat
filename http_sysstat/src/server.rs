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

use http_sysstat_pluginlib::stats_collector::{DateFormat, StatsCollector, StatsConfig};
use crate::config::Config;

type Plugins = Arc<Mutex<Vec<Box<dyn StatsCollector>>>>;

fn get_json_stats(cfg: &StatsConfig, data: &ServerData) -> HttpResponse {
    let collectors = data.collectors.clone();
    let collectors_guard = collectors.lock().unwrap();
    HttpResponse::Ok().json(
        (*collectors_guard).iter()
        .map(|c| ((*c).name(), (*c).collect(&cfg).unwrap()))
        .collect::<HashMap<&'static str, Value>>()
    )
}

#[derive(Deserialize)]
struct IndexQuery {
    date_format: Option<DateFormat>,
    human_readable: Option<bool>
}

struct ServerData {
    collectors: Plugins,
    cfg: Config,
}

#[get("/")]
async fn index(info: Query<IndexQuery>, data: Data<ServerData>, req: HttpRequest) -> impl Responder {
    let cfg = StatsConfig {
        date_format: info.date_format.unwrap_or(DateFormat::Epoch),
        human_readable: info.human_readable.unwrap_or(false),
        query_other: serde_urlencoded::from_str(req.query_string()).unwrap(),
        plugin_config: data.cfg.plugin_config.clone(),
    };

    get_json_stats(&cfg, &data)
}

#[get("/test")]
async fn test(info: Query<IndexQuery>, data: Data<ServerData>, req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("test")
}

#[actix_rt::main]
pub async fn start_server(cfg: Config, plugins: Plugins) -> Result<()> {
    let addr = cfg.addr.clone();
    HttpServer::new(move || 
            App::new()
                .service(index)
                .service(test)
                .data(ServerData {
                    collectors: plugins.clone(),
                    cfg: cfg.clone(),
                })
        )
        .bind(addr)?
        .run()
        .await
}
