/// House controller
use crate::controller::{RoomConfig, RoomController};
use log::debug;
use rumqttc::v5::{AsyncClient, mqttbytes::QoS};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HouseConfig {
    pub rooms: Vec<RoomConfig>,
    pub boiler_sw: String,
}

#[derive(Debug)]
pub struct HouseController {
    pub rooms: Vec<RoomController>,
    boiler_sw: String,
}

impl HouseController {
    pub fn new(config: HouseConfig) -> HouseController {
        let mut rooms = Vec::new();

        for cfg in config.rooms {
            rooms.push(RoomController::new(cfg));
        }

        HouseController {
            rooms: rooms,
            boiler_sw: config.boiler_sw,
        }
    }

    // TODO: find an alternative to passing in the MQTT client as it breaks the abstraction.
    pub async fn tick(&mut self, client: &AsyncClient) {
        for room in &mut self.rooms {
            room.tick(client).await;
            debug!("{}", room);
        }

        // debug!("{}", room);

        // client
        //     .publish(
        //         format!("zigbee2mqtt/{}/set/state_l1", self.boiler_sw),
        //         QoS::AtLeastOnce,
        //         true,
        //         "ONNN".as_bytes(),
        //     )
        //     .await
        //     .unwrap();
    }
}

#[cfg(test)]
mod tests {

    use crate::controller::RoomConfig;
    use crate::schedule::Schedule;

    use super::*;

    #[test]
    fn test_new() {
        let house_cfg = HouseConfig {
            rooms: vec![RoomConfig {
                name: "Test".into(),
                temp_sensor: "Test".into(),
                trv_device: "Test".into(),
                schedule: Schedule::new(),
            }],
            boiler_sw: "boiler".into(),
        };
        HouseController::new(house_cfg);
    }
}
