/// Server
use crate::config::Config;
use crate::house::HouseController;
use log::{debug, info};
use rumqttc::v5::EventLoop;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions, mqttbytes::QoS};
use serde_json::Value;
use std::error::Error;
use std::time::Duration;

pub struct Server {
    client: AsyncClient,
    eventloop: EventLoop,
    house: HouseController,
}

impl Server {
    pub fn new() -> Server {
        let config = Config::load().unwrap();

        let mut mqttoptions = MqttOptions::new("chimum", config.server, config.port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_max_packet_size(Some(config.max_packet_size));
        mqttoptions.set_credentials(config.username, config.password);

        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

        Server {
            client: client,
            eventloop: eventloop,
            house: HouseController::new(config.house),
        }
    }

    pub async fn event_handle(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        // Performs pattern matching on the retrieved event to determine its type
        match &event {
            Event::Incoming(Incoming::Publish(packet)) => {
                let topic = String::from_utf8(packet.topic.to_vec())?;
                let parts: Vec<&str> = topic.split("/").collect::<Vec<&str>>();

                if parts.len() > 2 {
                    return Ok(());
                }

                let device = parts[1];

                info!("Device: {}", device);

                for room in &mut self.house.rooms {
                    if device != room.temp_sensor {
                        continue;
                    }

                    let payload = String::from_utf8(packet.payload.to_vec())?;
                    let v: Value = serde_json::from_str(&payload)?;

                    debug!("  - {:?}", v);

                    room.update_temp(v["temperature"].as_f32().unwrap());

                    break;
                }
            }
            _ => {}
        }
        return Ok(());
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.client
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

        // Poll the event loop
        loop {
            // Waits for and retrieves the next event in the event loop.
            let res = self.eventloop.poll().await;

            match &res {
                Ok(event) => {
                    self.event_handle(event.clone()).await.unwrap();
                }
                Err(e) => {
                    println!("Error = {e:?}");
                    return Ok(());
                }
            }

            /*
            Should be time based - although this works for the moment assuming regular ZigBee
            messages are coming in.
            */
            self.house.tick(&self.client).await;
        }
    }
}
