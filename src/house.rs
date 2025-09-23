/// House controller
use crate::controller::{RoomConfig, RoomController};
use log::{debug, info};
use rumqttc::v5::{AsyncClient, mqttbytes::QoS};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub fn update_sensors(&mut self, device: &str, payload: &Value) {
        for room in &mut self.rooms {
            room.update_sensors(device, payload);
        }
    }

    // TODO: find an alternative to passing in the MQTT client as it breaks the abstraction.
    pub async fn tick(&mut self, client: &AsyncClient) {
        let mut heat_demand = false;

        for room in &mut self.rooms {
            room.tick(client).await;
            debug!("{}", room);

            if room.current_heat_demand() {
                heat_demand = true;
            };
        }

        debug!("Boiler: {}", heat_demand);
        return;
        let _ = match client
            .publish(
                format!("zigbee2mqtt/{}/set/state_l1", self.boiler_sw),
                QoS::AtLeastOnce,
                true,
                if heat_demand {
                    "ON".as_bytes()
                } else {
                    "OFF".as_bytes()
                },
            )
            .await
        {
            Err(e) => {
                log::error!("Boilar actuate error: {}", e)
            }
            _ => {
                info!("Sent")
            }
        };
    }
}

#[cfg(test)]
mod tests {

    use crate::controller::RoomConfig;
    use crate::schedule::Schedule;
    use crate::sensor::FloatSensorConfig;

    use super::*;

    #[test]
    fn test_new() {
        let house_cfg = HouseConfig {
            rooms: vec![RoomConfig {
                name: "Test".into(),
                temp_sensor: FloatSensorConfig {
                    device: "Test TH".into(),
                    field: "temperature".into(),
                },
                trv_device: "Test".into(),
                schedule: Schedule::new(),
                enable: true,
            }],
            boiler_sw: "boiler".into(),
        };
        HouseController::new(house_cfg);
    }
}
