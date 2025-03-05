use std::time;

use log::error;
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
    P115,
}

impl DeviceType {
    pub async fn get_handler(
        &self,
        client: impl Into<tapo::ApiClient>,
        ip: impl Into<String>,
    ) -> Result<PlugEnergyMonitoringHandler, error::Error> {
        let tapo_client = client.into().with_timeout(time::Duration::from_secs(10));
        match self {
            Self::P110 => Ok(tapo_client.p110(ip).await?),
            Self::P115 => Ok(tapo_client.p115(ip).await?),
        }
    }
}

#[derive(Clone)]
pub struct Credential {
    username: String,
    password: String,
}

impl From<config::Credential> for Credential {
    fn from(value: config::Credential) -> Self {
        return Self {
            username: value.username,
            password: value.password,
        };
    }
}

impl Into<tapo::ApiClient> for Credential {
    fn into(self) -> tapo::ApiClient {
        tapo::ApiClient::new(self.username, self.password)
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
    client: Option<PlugEnergyMonitoringHandler>,
    device: DeviceType,
    ip: String,
    credential: Credential,
}

impl Collector {
    pub fn new(
        metrics: &Metrics,
        device: impl Into<Labels>,
        credential: impl Into<Credential>,
    ) -> Self {
        let label = device.into();

        Self {
            energy_usage: metrics.energy_usage.get_or_create(&label).clone(),
            device_on: metrics.device_on.get_or_create(&label).clone(),
            request_fail: metrics.request_fail.get_or_create(&label).clone(),
            client: None,
            device: label.device,
            ip: label.ip,
            credential: credential.into(),
        }
    }

    pub async fn get_client(&mut self) -> Result<(), error::Error> {
        if let Some(_) = self.client {
            Ok(())
        } else {
            match self
                .device
                .get_handler(self.credential.to_owned(), self.ip.clone())
                .await
            {
                Ok(client) => {
                    self.client = Some(client);
                    Ok(())
                }
                Err(error) => {
                    error!("get client fail {}", error);
                    Err(error)
                }
            }
        }
    }

    pub async fn collect(&mut self) {
        let client = match self.get_client().await {
            Ok(_) => self.client.as_ref().unwrap(),
            Err(_) => return,
        };

        let mut reset_client = false;

        match client.get_energy_usage().await {
            Ok(energy_usage) => {
                self.energy_usage.set(energy_usage.current_power as i64);
            }
            Err(_) => {
                reset_client = true;
                self.request_fail.inc();
            }
        }

        match client.get_device_info().await {
            Ok(status) => {
                if status.device_on {
                    self.device_on.set(1);
                } else {
                    self.device_on.set(0);
                }
            }
            Err(_) => {
                reset_client = true;
                self.request_fail.inc();
            }
        }

        if reset_client {
            self.client = None
        }
    }
}
