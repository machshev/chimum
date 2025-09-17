/// House controller
use crate::controller::{RoomConfig, RoomController};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HouseConfig {
    pub rooms: Vec<RoomConfig>,
}

impl HouseConfig {
    pub fn new() -> HouseConfig {
        HouseConfig { rooms: Vec::new() }
    }
}

#[derive(Debug)]
pub struct HouseController {
    pub rooms: Vec<RoomController>,
}

impl HouseController {
    pub fn new(config: HouseConfig) -> HouseController {
        let mut rooms = Vec::new();

        for cfg in config.rooms {
            rooms.push(RoomController::new(cfg));
        }

        HouseController { rooms: rooms }
    }
}

#[cfg(test)]
mod tests {

    use crate::controller::RoomConfig;
    use crate::schedule::Schedule;

    use super::*;

    #[test]
    fn test_new() {
        let mut house_cfg = HouseConfig::new();
        house_cfg.rooms.push(RoomConfig {
            name: "Test".into(),
            temp_sensor: "Test".into(),
            trv_device: "Test".into(),
            schedule: Schedule::new(),
        });
        HouseController::new(house_cfg);
    }
}
