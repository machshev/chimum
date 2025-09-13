use crate::{config::HouseConfig, controller::RoomController};

/// House controller

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
    use crate::config::{RoomConfig, Schedule};

    use super::*;

    #[test]
    fn test_new() {
        let mut house_cfg = HouseConfig::new();
        house_cfg.rooms.push(RoomConfig {
            name: "Test".into(),
            schedule: Schedule {},
        });
        HouseController::new(house_cfg);
    }
}
