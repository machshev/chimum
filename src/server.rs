/// Server
use crate::config::Config;
use crate::house::HouseController;
use log::trace;
use rumqttc::v5::EventLoop;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions};
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::{task, time};

pub struct Server {
    client: Arc<Mutex<AsyncClient>>,
    eventloop: EventLoop,
    house: Arc<Mutex<HouseController>>,
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
            client: Arc::new(Mutex::new(client)),
            eventloop: eventloop,
            house: Arc::new(Mutex::new(HouseController::new(config.house))),
        }
    }

    pub async fn event_handle(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        // Performs pattern matching on the retrieved event to determine its type
        match &event {
            Event::Incoming(Incoming::Publish(packet)) => {
                let topic = String::from_utf8(packet.topic.to_vec())?;
                let payload = String::from_utf8(packet.payload.to_vec())?;
                let parts: Vec<&str> = topic.split("/").collect::<Vec<&str>>();

                // Only read status messages, and ignore controller status
                if parts.len() > 2 || parts[1] == "bridge" {
                    return Ok(());
                }

                trace!("raw: {:?} = {:?}", topic, payload);

                let device = parts[1];
                let v: Value = serde_json::from_str(&payload)?;

                {
                    let mut house = self.house.lock().await;
                    house.update_sensors(device, &v);
                }
            }
            _ => {}
        }
        return Ok(());
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let client = self.client.lock().await;
            client.subscribe("zigbee2mqtt/#", QoS::AtMostOnce).await?;
        }

        // Clone Arc for the task
        let client_task = Arc::clone(&self.client);
        let house_task = Arc::clone(&self.house);

        task::spawn(async move {
            loop {
                let mut house = house_task.lock().await;
                let client = client_task.lock().await;
                house.tick(&*client).await;

                time::sleep(Duration::from_secs(5)).await;
            }
        });

        // Poll the event loop
        loop {
            // Waits for and retrieves the next event in the event loop.
            let res = self.eventloop.poll().await;

            match &res {
                Ok(event) => match self.event_handle(event.clone()).await {
                    Err(e) => {
                        println!("Error = {e:?}");
                    }
                    _ => {}
                },
                Err(e) => {
                    println!("Error = {e:?}");
                    return Ok(());
                }
            }
        }
    }
}
