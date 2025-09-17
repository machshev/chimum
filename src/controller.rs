/// Controller for room temperature
use log::info;

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
    pub config: RoomConfig,
    temp: f64,
    setpoint_on: f64,
    setpoint_off: f64,
    heat_demand: bool,
}

impl RoomController {
    fn recalculate(&mut self) {
        if self.heat_demand {
            self.heat_demand = self.temp < self.setpoint_off;
        } else {
            self.heat_demand = self.temp < self.setpoint_on;
        };
    }

    pub fn new(config: RoomConfig) -> RoomController {
        RoomController {
            config: config,
            temp: 0.0,
            setpoint_on: 0.0,
            setpoint_off: 0.0,
            heat_demand: false,
        }
    }

    pub fn update_temp(&mut self, temp: f64) {
        info!("Updating {} temp {}", self.config.name, temp);
        self.temp = temp;
        self.recalculate()
    }

    pub fn current_temp(&self) -> f64 {
        self.temp
    }

    pub fn update_setpoint(&mut self, setpoint: f64, hysteresis: f64) {
        self.setpoint_on = setpoint - (hysteresis / 2.0);
        self.setpoint_off = setpoint + (hysteresis / 2.0);
        self.recalculate()
    }

    pub fn current_setpoint_on(&self) -> f64 {
        self.setpoint_on
    }

    pub fn current_setpoint_off(&self) -> f64 {
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
