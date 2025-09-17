/// Server
use crate::config::Config;
use crate::house::HouseController;
use log::{debug, info};
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions, mqttbytes::QoS};
use serde_json::Value;
use std::error::Error;
use std::time::Duration;

pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;

    let mut mqttoptions = MqttOptions::new("chimum", config.server, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(Some(config.max_packet_size));
    mqttoptions.set_credentials(config.username, config.password);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .subscribe("zigbee2mqtt/#", QoS::AtMostOnce)
        .await
        .unwrap();

    // client
    //     .publish(
    //         "zigbee2mqtt/FRIENDLY_NAME/set/state",
    //         QoS::AtLeastOnce,
    //         true,
    //         "ON".as_bytes(),
    //     )
    //     .await
    //     .unwrap();

    let mut house = HouseController::new(config.house);

    // Poll the event loop
    loop {
        // Waits for and retrieves the next event in the event loop.
        let event = eventloop.poll().await;
        // Performs pattern matching on the retrieved event to determine its type
        match &event {
            Ok(v) => {
                if let Event::Incoming(Incoming::Publish(packet)) = v {
                    let topic = String::from_utf8(packet.topic.to_vec())?;
                    let parts: Vec<&str> = topic.split("/").collect::<Vec<&str>>();

                    if parts.len() > 2 {
                        continue;
                    }

                    let device = parts[1];

                    info!("Device: {}", device);

                    for room in &mut house.rooms {
                        if device != room.config.temp_sensor {
                            continue;
                        }

                        let payload = String::from_utf8(packet.payload.to_vec())?;
                        let v: Value = serde_json::from_str(&payload)?;

                        debug!("  - {:?}", v);

                        room.update_temp(v["temperature"].as_f64().unwrap());

                        break;
                    }
                };
            }
            Err(e) => {
                println!("Error = {e:?}");
                return Ok(());
            }
        }

        house.tick();
    }
}
