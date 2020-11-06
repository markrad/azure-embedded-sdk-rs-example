extern crate azure_embedded_sdk_rs as azrs;
extern crate azure_embedded_sdk_sys as azsys;
extern crate base64;
extern crate hmac_sha256;
extern crate paho_mqtt as mqtt;

use regex::Regex;
use std::time;
use std::time::{SystemTime, UNIX_EPOCH};
use std::env;
use std::process;
use std::thread;

unsafe extern "C" fn callback() {
    process::abort();
}

fn main() {
    azrs::precondition_failed_set_callback(Option::Some(callback));

    let connection_string = env::var("AZ_IOT_CONNECTION_STRING").expect("Connection string not found in environment");
    let certificate_name = env::var("AZ_IOT_ROOT_CERTIFICATE").expect("Root cerificate file name not found in environment");

    let host_name_re: Regex = Regex::new(r"(?i)HostName=([^;]*)").unwrap();
    let device_id_re: Regex = Regex::new(r"(?i)DeviceId=([^;]*)").unwrap();
    let shared_access_key_re: Regex = Regex::new(r"(?i)SharedAccessKey=([^;]*)").unwrap();
    let host_name = host_name_re.captures(&connection_string).expect("Invalid connection string").get(1).map_or("", |m| m.as_str());
    let device_id = device_id_re.captures(&connection_string).expect("Invalid connection string").get(1).map_or("", |m| m.as_str());
    let shared_access_key = shared_access_key_re.captures(&connection_string).expect("Invalid connection string").get(1).map_or("", |m| m.as_str());

    if ! std::path::Path::new(&certificate_name).exists() {
        println!("Root certificate file does not exist");
        process::exit(4);
    }

    let options = azrs::HubClientOptions::default_new();

    let client = azrs::HubClient::new(&host_name, &device_id, Option::Some(options)).expect("Failed to create HubClient");
    let mqtt_client_id = client.get_client_id().expect("Failed to get MQTT client id");
    let mqtt_user_name = client.get_user_name().expect("Failed to get MQTT user name");
    let publish_topic = client.get_client_telemetry_publish_topic(Option::None).expect("Failed to get topic");
    let ttl = 120;

    let (mut mqtt_password, mut expiry_time) = get_password(&client, ttl, &shared_access_key);

    println!("Connection Information:");
    println!("\tHostname = {}", host_name);
    println!("\tDevice Id = {}", device_id);
    println!("\tRoot certificate file location = {}", certificate_name);
    println!("\tMQTT client Id = {}", mqtt_client_id);
    println!("\tMQTT user Name = {}", mqtt_user_name);
    println!("\tMQTT password = {}", mqtt_password);
    println!("\tMQTT telemetry publish topic = {}", publish_topic);

    let uri = "ssl://".to_string() + host_name + ":8883";
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(uri)
        .client_id(mqtt_client_id)
        .persistence(mqtt::PersistenceType::None)
        .finalize();

    let mqtt_client = mqtt::Client::new(create_opts).expect("Failed to create MQTT client");

    // let ssl_opts = mqtt::SslOptionsBuilder::new()
    //     .trust_store(&certificate_name)
    //     .finalize();

    let connect_opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(&mqtt_user_name)
        .password(&mqtt_password)
        .ssl_options(mqtt::SslOptionsBuilder::new().trust_store(&certificate_name).finalize())
        .automatic_reconnect(time::Duration::new(1, 0), time::Duration::new(60 * 60, 0))
        .finalize();

    println!("Connecting to server");

    if let Err(e) = mqtt_client.connect(connect_opts) {
        println!("Failed to connect to server: {}", e);
        process::exit(4);
    }

    println!("Connected");

    let mut message: mqtt::Message;
    let message_count = 300;
    let mut message_tracker = 0;
    let mut loop_counter = -1;

    while message_tracker < message_count {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Could not get time").as_secs();
        if expiry_time - now < ttl / 100 * 80 {
            println!("Reauthenticating");
            mqtt_client.disconnect(mqtt::DisconnectOptions::new()).expect("Failed to disconnect");
            let parts = get_password(&client, ttl, &shared_access_key);
            mqtt_password = parts.0;
            expiry_time = parts.1;

            let connect_opts = mqtt::ConnectOptionsBuilder::new()
                .user_name(&mqtt_user_name)
                .password(&mqtt_password)
                .ssl_options(mqtt::SslOptionsBuilder::new().trust_store(&certificate_name).finalize())
                .automatic_reconnect(time::Duration::new(1, 0), time::Duration::new(60 * 60, 0))
                .finalize();

            if let Err(e) = mqtt_client.connect(connect_opts) {
                println!("Failed to connect to server: {}", e);
                process::exit(4);
            }
        }

        loop_counter += 1;

        if loop_counter % 100 == 0 {
            message = mqtt::MessageBuilder::new()
                .topic(&publish_topic)
                .payload(format!("Rust Message #{}", message_tracker))
                .qos(1)
                .finalize();
            println!("Sending message #{}", message_tracker);
            match mqtt_client.publish(message) {
                Ok(_n) => println!("Sent"),
                Err(err) => {
                    println!("Send failed {}", err);
                    process::exit(4);
                }
            }
            message_tracker += 1;
        }
        thread::sleep(time::Duration::from_millis(10));
    }

    mqtt_client.disconnect(mqtt::DisconnectOptions::new()).expect("Failed to disconnect");

    println!("done");
}

fn get_password(client: &azrs::HubClient, ttl: u64, shared_access_key: &str) -> (String, u64) {
    let epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Could not get time").as_secs() + ttl as u64;
    let signature = client.get_sas_signature(epoch).expect("Failed to get SAS signature");
    let decoded_key = base64::decode(shared_access_key).expect("Shared access key is not valid Base 64");
    let sas = base64::encode(hmac_sha256::HMAC::mac(&signature, &decoded_key));
    let password = client.get_sas_password(epoch, &sas).expect("Failed to get password");

    (password, epoch)
}
