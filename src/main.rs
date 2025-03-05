use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use core::time;
use device::Collector;
use env_logger;
use log::info;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

mod config;
mod device;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = config::Config::load_file("config.yml")?;

    let metrics = device::Metrics::default();
    let mut registry = Registry::default();
    registry.register(
        "tapo_energy_usage",
        "current power in mW",
        metrics.energy_usage.clone(),
    );
    registry.register(
        "tapo_device_on",
        "current switch status",
        metrics.device_on.clone(),
    );
    registry.register(
        "tapo_request_fail",
        "device request fail",
        metrics.request_fail.clone(),
    );

    let mut collectors = Vec::with_capacity(config.devices.len());

    for device in config.devices {
        let credential = config
            .credentials
            .iter()
            .find(|x| x.name == device.credential)
            .expect("credential not found");

        collectors.push(Arc::new(Mutex::new(device::Collector::new(
            &metrics,
            device.clone(),
            credential.to_owned(),
        ))));
    }

    // collector loop
    tokio::spawn(async move { collect(&mut collectors, config.interval).await });

    info!("string metrics server");

    let app = axum::Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(Arc::new(registry));

    let listener = TcpListener::bind("0.0.0.0:9200").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn metrics_handler(State(state): State<Arc<Registry>>) -> impl IntoResponse {
    let mut buffer = String::new();
    encode(&mut buffer, &state).unwrap();

    buffer
}

async fn collect(collectors: &mut Vec<Arc<Mutex<Collector>>>, interval: u64) {
    let mut timer = tokio::time::interval(time::Duration::from_secs(interval));
    loop {
        timer.tick().await;
        info!("collect device status");
        let mut set = tokio::task::JoinSet::new();

        for collector in collectors.iter_mut() {
            let collector = collector.clone();
            set.spawn(async move { collector.lock().await.collect().await });
        }
        set.join_all().await;
    }
}
