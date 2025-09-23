use log::debug;
/// Sensors
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait MQTTSensor {
    fn device_name(&self) -> &str;
    fn update(&mut self, payload: &Value);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FloatSensorConfig {
    pub device: String,
    pub field: String,
}

#[derive(Debug)]
pub struct FloatSensor {
    device: String,
    field: String,
    value: f32,
    valid: bool,
}

impl MQTTSensor for FloatSensor {
    fn device_name(&self) -> &str {
        &self.device
    }

    fn update(&mut self, payload: &Value) {
        self.value = payload[&self.field].as_f64().unwrap() as f32;
        self.valid = true;

        debug!("Device: {} = {}", self.device, self.value);
    }
}

impl FloatSensor {
    pub fn new(config: FloatSensorConfig) -> FloatSensor {
        FloatSensor {
            device: config.device,
            field: config.field,
            value: 0.0,
            valid: false,
        }
    }

    #[cfg(test)]
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
        self.valid = true;
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }
}
