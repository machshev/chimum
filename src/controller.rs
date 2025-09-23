use std::fmt;

/// Controller for room temperature
use chrono::{Datelike, Local, Timelike};

use rumqttc::v5::{AsyncClient, mqttbytes::QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schedule::Schedule;
use crate::sensor::MQTTSensor;
use crate::sensor::{FloatSensor, FloatSensorConfig};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RoomConfig {
    pub name: String,
    pub enable: bool,
    pub temp_sensor: FloatSensorConfig,
    pub trv_device: String,
    pub schedule: Schedule,
}

#[derive(Debug)]
pub struct RoomController {
    name: String,
    temp_sensor: FloatSensor,
    trv_device: String,
    schedule: Schedule,
    setpoint_on: f32,
    setpoint_off: f32,
    heat_demand: bool,
    pub enable: bool,
}

impl fmt::Display for RoomController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Room({}, [{}|{}]  {}°C -- {})",
            self.name,
            self.setpoint_on,
            self.setpoint_off,
            self.temp_sensor.get_value(),
            if self.heat_demand { "HEAT" } else { "OFF" },
        )
    }
}

impl RoomController {
    pub fn new(config: RoomConfig) -> RoomController {
        RoomController {
            setpoint_on: 0.0,
            setpoint_off: 0.0,
            heat_demand: false,
            name: config.name,
            temp_sensor: FloatSensor::new(config.temp_sensor),
            trv_device: config.trv_device,
            schedule: config.schedule,
            enable: config.enable,
        }
    }

    pub fn update_sensors(&mut self, device: &str, payload: &Value) {
        if device != self.temp_sensor.device_name() {
            return;
        }

        self.temp_sensor.update(payload);
    }

    fn recalculate(&mut self) {
        if !self.enable {
            self.heat_demand = false;
            return;
        };

        if self.heat_demand {
            self.heat_demand = self.temp_sensor.get_value() < self.setpoint_off;
        } else {
            self.heat_demand = self.temp_sensor.get_value() < self.setpoint_on;
        };
    }

    fn update_time<T: Datelike + Timelike>(&mut self, now: T) {
        let day = now.weekday().num_days_from_sunday();
        let hour = now.hour();
        let min = now.minute();

        let setpoint = self.schedule.get_setpoint(
            day.try_into().unwrap(),
            hour.try_into().unwrap(),
            min.try_into().unwrap(),
        );

        self.update_setpoint(setpoint, 2.0);
    }

    pub async fn tick(&mut self, client: &AsyncClient) {
        let now = Local::now();

        self.update_time(now);

        if !self.enable {
            return;
        }

        // Only transmit if there is a change
        let _ = client
            .publish(
                format!(
                    "zigbee2mqtt/{}/set/current_heating_setpoint",
                    self.trv_device
                ),
                QoS::AtLeastOnce,
                true,
                if self.heat_demand {
                    "45".as_bytes()
                } else {
                    "5".as_bytes()
                },
            )
            .await;
    }

    pub fn update_setpoint(&mut self, setpoint: f32, hysteresis: f32) {
        self.setpoint_on = setpoint - (hysteresis / 2.0);
        self.setpoint_off = setpoint + (hysteresis / 2.0);
        self.recalculate()
    }

    #[cfg(test)]
    fn current_setpoint_on(&self) -> f32 {
        self.setpoint_on
    }

    #[cfg(test)]
    fn current_setpoint_off(&self) -> f32 {
        self.setpoint_off
    }

    pub fn current_heat_demand(&self) -> bool {
        self.heat_demand
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use crate::schedule::Schedule;

    use super::*;

    fn test_config() -> RoomConfig {
        RoomConfig {
            name: "Test".into(),
            temp_sensor: FloatSensorConfig {
                device: "Test TH".into(),
                field: "temperature".into(),
            },
            trv_device: "Test TRV".into(),
            schedule: Schedule::new(),
            enable: true,
        }
    }

    fn test_config_with_schedule(schedule_json: &str) -> RoomConfig {
        let mut config = test_config();

        let schedule: Schedule = from_str(schedule_json).unwrap();

        config.schedule = schedule;

        config
    }

    #[test]
    fn test_update_setpoint() {
        let mut controller = RoomController::new(test_config());

        assert_eq!(controller.current_setpoint_on(), 0.0);
        assert_eq!(controller.current_setpoint_off(), 0.0);

        controller.update_setpoint(20.0, 1.0);

        assert_eq!(controller.current_setpoint_on(), 19.5);
        assert_eq!(controller.current_setpoint_off(), 20.5);
    }

    #[test]
    fn test_heat_demand() {
        let mut controller = RoomController::new(test_config());

        assert_eq!(controller.current_heat_demand(), false, "init no demand");

        controller.update_setpoint(20.0, 1.0);

        assert_eq!(controller.current_heat_demand(), true, "heat demand");

        controller.temp_sensor.set_value(20.6);
        controller.recalculate();

        assert_eq!(controller.current_heat_demand(), false, "room heated");

        controller.temp_sensor.set_value(19.5);
        controller.recalculate();

        assert_eq!(
            controller.current_heat_demand(),
            false,
            "temp lowered within range"
        );

        controller.temp_sensor.set_value(19.4);
        controller.recalculate();

        assert_eq!(controller.current_heat_demand(), true, "room reheat");
    }

    #[test]
    fn test_schedule() {
        let schedule_json = r#"
            [
                [7, 0, 20.0],
                [7, 30, 17.0],
                [17, 30, 18.0],
                [21, 0, 20.0],
                [22, 0, 18.0]
            ]
        "#;
        let mut controller = RoomController::new(test_config_with_schedule(schedule_json));
    }
}
