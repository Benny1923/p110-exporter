use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::encoding::EncodeLabelValue;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use serde::{Deserialize, Serialize};
use tapo::PlugEnergyMonitoringHandler;

use crate::config;
use crate::error;

#[non_exhaustive]
#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelValue, Serialize, Deserialize)]
pub enum DeviceType {
    P110,
}

impl DeviceType {
    pub async fn get_handler(
        &self,
        client: tapo::ApiClient,
        ip: impl Into<String>,
    ) -> Result<PlugEnergyMonitoringHandler, error::Error> {
        match self {
            Self::P110 => Ok(client.p110(ip).await?),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct Labels {
    pub device: DeviceType,
    pub name: String,
    pub ip: String,
}

impl From<config::Device> for Labels {
    fn from(value: config::Device) -> Self {
        Self {
            device: value.device,
            name: value.name,
            ip: value.ip,
        }
    }
}

#[derive(Debug, Default)]
pub struct Metrics {
    pub energy_usage: Family<Labels, Gauge>, // current power in mW
    pub device_on: Family<Labels, Gauge>,    // 0 is off, 1 is on

    pub request_fail: Family<Labels, Counter>,
}


pub struct Collector {
    // metrics
    energy_usage: Gauge,
    device_on: Gauge,
    request_fail: Counter,

    // client
    client: PlugEnergyMonitoringHandler,
}

impl Collector {
    pub async fn new(
        metrics: &Metrics,
        device: &config::Device,
        credential: &config::Credential,
    ) -> Result<Self, error::Error> {
        let label = device.clone().into();

        let result = Self {
            energy_usage: metrics.energy_usage.get_or_create(&label).clone(),
            device_on: metrics.device_on.get_or_create(&label).clone(),
            request_fail: metrics.request_fail.get_or_create(&label).clone(),

            client: device.device.get_handler(credential.clone().into(), device.ip.clone()).await?
        };

        Ok(result)
    }

    pub async fn collect(&self) {
        if let Ok(energy_usage) = self.client.get_energy_usage().await {
            self.energy_usage.set(energy_usage.current_power as i64);
        } else {
            self.request_fail.inc();
        }

        if let Ok(status) = self.client.get_device_info().await {
            if status.device_on {
                self.device_on.set(1);
            } else {
                self.device_on.set(0);
            }
        } else {
            self.request_fail.inc();
        }
    }
}
