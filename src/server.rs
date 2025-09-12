/// Server
use super::config::Config;
use rumqttc::v5::{AsyncClient, Event, Incoming, MqttOptions, mqttbytes::QoS};
use std::error::Error;
use std::time::Duration;
use tokio::{task, time};

pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;

    let mut mqttoptions = MqttOptions::new("chimum", config.server, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_max_packet_size(Some(config.max_packet_size));
    mqttoptions.set_credentials(config.username, config.password);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
        requests(client).await;
        time::sleep(Duration::from_secs(3)).await;
    });

    // Poll the event loop
    loop {
        // Waits for and retrieves the next event in the event loop.
        let event = eventloop.poll().await;
        // Performs pattern matching on the retrieved event to determine its type
        match &event {
            Ok(v) => {
                if let Event::Incoming(Incoming::Publish(packet)) = v {
                    println!("{:?}", packet.topic);
                };
            }
            Err(e) => {
                println!("Error = {e:?}");
                return Ok(());
            }
        }
    }
}

async fn requests(client: AsyncClient) {
    /*
     * Used to subscribe to a specific topic ("hello/world") on the MQTT server,
     * specifying the Quality of Service (QoS) as AtMostOnce, indicating at most
     * once message delivery.
     */
    client.subscribe("#", QoS::AtMostOnce).await.unwrap();
}
