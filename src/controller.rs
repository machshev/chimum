use std::fmt;

/// Controller for room temperature
use chrono::{Datelike, Local, Timelike};
use log::info;

use rumqttc::v5::{AsyncClient, mqttbytes::QoS};
use serde::{Deserialize, Serialize};

use crate::schedule::Schedule;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RoomConfig {
    pub name: String,
    pub temp_sensor: String,
    pub trv_device: String,
    pub schedule: Schedule,
}

#[derive(Debug)]
pub struct RoomController {
    name: String,
    pub temp_sensor: String,
    trv_device: String,
    schedule: Schedule,
    temp: f32,
    setpoint_on: f32,
    setpoint_off: f32,
    heat_demand: bool,
}

impl fmt::Display for RoomController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Room({}, [{}|{}]  {}°C -- {})",
            self.name, self.setpoint_on, self.setpoint_off, self.temp, self.heat_demand,
        )
    }
}

impl RoomController {
    pub fn new(config: RoomConfig) -> RoomController {
        RoomController {
            temp: 0.0,
            setpoint_on: 0.0,
            setpoint_off: 0.0,
            heat_demand: false,
            name: config.name,
            temp_sensor: config.temp_sensor,
            trv_device: config.trv_device,
            schedule: config.schedule,
        }
    }

    fn recalculate(&mut self) {
        if self.heat_demand {
            self.heat_demand = self.temp < self.setpoint_off;
        } else {
            self.heat_demand = self.temp < self.setpoint_on;
        };
    }

    pub async fn tick(&mut self, client: &AsyncClient) {
        let now = Local::now();

        let day = now.weekday().num_days_from_sunday();
        let hour = now.hour();
        let min = now.minute();

        let setpoint = self.schedule.get_setpoint(
            day.try_into().unwrap(),
            hour.try_into().unwrap(),
            min.try_into().unwrap(),
        );

        self.update_setpoint(setpoint, 2.0);

        client
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
            .await
            .unwrap();
    }

    pub fn update_temp(&mut self, temp: f32) {
        info!("Updating {} temp {}", self.name, temp);
        self.temp = temp;
        self.recalculate()
    }

    pub fn current_temp(&self) -> f32 {
        self.temp
    }

    pub fn update_setpoint(&mut self, setpoint: f32, hysteresis: f32) {
        self.setpoint_on = setpoint - (hysteresis / 2.0);
        self.setpoint_off = setpoint + (hysteresis / 2.0);
        self.recalculate()
    }

    pub fn current_setpoint_on(&self) -> f32 {
        self.setpoint_on
    }

    pub fn current_setpoint_off(&self) -> f32 {
        self.setpoint_off
    }

    pub fn current_heat_demand(&self) -> bool {
        self.heat_demand
    }
}

#[cfg(test)]
mod tests {
    use crate::schedule::Schedule;

    use super::*;

    fn test_config() -> RoomConfig {
        RoomConfig {
            name: "Test".into(),
            temp_sensor: "Test TH".into(),
            trv_device: "Test TRV".into(),
            schedule: Schedule::new(),
        }
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

        controller.update_temp(20.6);

        assert_eq!(controller.current_heat_demand(), false, "room heated");

        controller.update_temp(19.5);

        assert_eq!(
            controller.current_heat_demand(),
            false,
            "temp lowered within range"
        );

        controller.update_temp(19.4);

        assert_eq!(controller.current_heat_demand(), true, "room reheat");
    }
}
