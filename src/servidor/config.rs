use serde::Deserialize;
use std::env;
use std::fs;

pub fn helper() {
    println!("Helper function from utils.rs!");
}
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct ServerConfig {
    password: String,
    host: String,
    port1: String,
    port2: String,
}

pub fn get_configuration() ->Config { //} Result<(), Box<dyn std::error::Error>> {
    // Read the JSON file
    let config_file = fs::read_to_string("serv_config.json").unwrap();

    // Deserialize the JSON into the Config struct
    let config: Config = serde_json::from_str(&config_file).unwrap();


    println!("Configuration: {:?}", config);
    config
}


pub fn print_path() {
    println!("Current directory: {:?}", env::current_dir().unwrap());
    let config_file = fs::read_to_string("config.json").expect("Failed to read config.json. Please ensure the file exists.");
    println!("Config file content: {}", config_file);
}

pub fn get_password(config: &Config) ->String{
    config.server.password.clone()
}

pub fn get_hostip(config: &Config) ->String{
    config.server.host.clone()
}

pub fn get_port1(config: &Config) ->String{
    config.server.port1.clone()
}

pub fn get_port2(config: &Config) ->String{
    config.server.port2.clone()
}

