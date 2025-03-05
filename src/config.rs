use std::fs::File;

use crate::error;
use crate::device;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub devices: Vec<Device>,
    pub credentials: Vec<Credential>,

    pub interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub ip: String,
    #[serde(alias = "type")]
    pub device: device::DeviceType,
    pub credential: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub name: String,
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn load_file(path: &str) -> Result<Self, error::Error> {

        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => return Err(error::Error::Io(err)),
        };

        let config: Self = match serde_yaml::from_reader(&mut file) {
            Ok(config) => config,
            Err(err) => return Err(error::Error::Parse(err)),
        };

        config.validate()?;

        Ok(config)
    }


    pub fn validate(&self) -> Result<(), error::Error> {

        let credentials: Vec<&String> = self.credentials.iter().map(|c| &c.name).collect();
        
        for device in self.devices.iter() {
            if !credentials.contains(&&device.credential) {
                return Err(error::Error::InvalidConfig);
            }
        }

        Ok(())   
    }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    use super::*;

    const PLAINTEXT: &str = r#"
devices:
  - name: device_name
    ip: 192.168.1.2
    type: P110
    credential: default

credentials:
  - name: default
    username: user@example.com
    password: test

interval: 300
"#;

    #[test]
    fn test_config() {
        let mut file = File::create("/tmp/test.yml").unwrap();

        file.write_all(PLAINTEXT.as_bytes()).unwrap();
        
        file.flush().unwrap();

        let config = Config::load_file("/tmp/test.yml").unwrap();

        // check all fields are loaded correctly
        assert_eq!(config.devices[0].name, "device_name");
        assert_eq!(config.devices[0].ip, "192.168.1.2");
        assert!(matches!(config.devices[0].device, device::DeviceType::P110));
        assert_eq!(config.devices[0].credential, "default");

        assert_eq!(config.credentials[0].name, "default");
        assert_eq!(config.credentials[0].username, "user@example.com");
        assert_eq!(config.credentials[0].password,"test");

        assert_eq!(config.interval, 300);
    }

}