use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use device::Collector;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use core::time;
use std::sync::Arc;
use tokio::net::TcpListener;

mod config;
mod device;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        collectors.push(
            device::Collector::new(
                &metrics,
                &device,
                config
                    .credentials
                    .iter()
                    .find(|&x| x.name == device.credential)
                    .expect("credential not found"),
            )
            .await?,
        );
    }

    tokio::spawn(async move {
        collect(collectors, config.interval).await;
    });

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

async fn collect(collectors: Vec<Collector>, interval: u64) {
    loop {
        for collector in collectors.iter() {
            collector.collect().await;
        }
        tokio::time::sleep(time::Duration::from_secs(interval)).await;
    }
}
