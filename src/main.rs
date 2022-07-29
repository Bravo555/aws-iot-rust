use clap::Parser;
use rumqttc::{AsyncClient, Key, MqttOptions, QoS, TlsConfiguration, Transport};
use std::time::Duration;
use tokio::{task, time};

#[derive(Parser, Debug)]
#[clap(about)]
struct Args {
    /// Path to client cert file
    #[clap(short = 'c', long, value_parser)]
    client_cert: String,
    /// Path to client private key file
    #[clap(short = 'k', long, value_parser)]
    client_private_key: String,
    /// Path to CA cert file
    #[clap(short = 'r', long, value_parser)]
    ca_cert: String,
}

#[tokio::main(worker_threads = 1)]
async fn main() {
    pretty_env_logger::init();

    let args = Args::parse();

    let client_cert_path = args.client_cert;
    let client_private_key_path = args.client_private_key;
    let ca_cert_path = args.ca_cert;

    let client_cert_pem = std::fs::read(client_cert_path).unwrap();

    let client_private_key_pem = std::fs::read(client_private_key_path).unwrap();

    let ca_cert_pem = std::fs::read(ca_cert_path).unwrap();

    let transport = Transport::Tls(TlsConfiguration::Simple {
        ca: ca_cert_pem,
        alpn: None,
        client_auth: Some((client_cert_pem, Key::RSA(client_private_key_pem))),
    });

    let mut mqttoptions = MqttOptions::new(
        "rust-client",
        "a2e6ahkpo54syc-ats.iot.eu-central-1.amazonaws.com",
        8883,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_transport(transport);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
        requests(client).await;
        time::sleep(Duration::from_secs(3)).await;
    });

    loop {
        let event = eventloop.poll().await;
        println!("{:?}", event);
    }
}

async fn requests(client: AsyncClient) {
    client
        .subscribe("hello/world", QoS::AtMostOnce)
        .await
        .unwrap();

    for i in 1..=10 {
        let payload = format!(r#"{{"message": "hello from rust!","sequence": {i} }}"#);
        client
            .publish("hello/world", QoS::AtMostOnce, false, payload)
            .await
            .unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }

    time::sleep(Duration::from_secs(120)).await;
}
