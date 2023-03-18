use std::fs;
use std::collections::HashMap;
use std::env;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfigFile {
    pub network: HashMap<String, Vec<u16>>,
    pub server: HashMap<String, u16>,
    pub settings: HashMap<String, u8>,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub update_port: u16,
    pub command_port: u16,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct ElevatorSettings {
    pub num_floors: u8,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    pub server: ServerConfig,
    pub settings: ElevatorSettings,
}

impl Config {
    pub fn get() -> Self {
        let file_path = "config.json";
        let fallback_file_path = "_config.json";
        let config_contents = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => {
                println!("No configuration file provided, using default settings...");
                fs::read_to_string(fallback_file_path).unwrap()
            },
        };
        let config_file: ConfigFile = serde_json::from_str(&config_contents).unwrap();
        let (elevnum, serverport) = parse_env_args(config_file.server["port"]);
        
        Config {
            network: NetworkConfig { 
                update_port: config_file.network["updatePorts"][elevnum as usize], 
                command_port: config_file.network["commandPorts"][elevnum as usize],
            },
            server: ServerConfig { 
                port: serverport,
            },
            settings: ElevatorSettings { 
                num_floors: config_file.settings["numFloors"], 
            },
        }
    }
}

fn parse_env_args(defaultport: u16) -> (u16, u16) {
    let (mut elevnum, mut serverport) = (0, defaultport);

    let args: Vec<String> = env::args().collect();
    for arg_pair in args.rchunks_exact(2) {
        match arg_pair[0].as_str() {
            "--elevnum" => {
                elevnum = match arg_pair[1].parse::<u16>() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("elevnum {} is not a number, skipping...", arg_pair[1]);
                        elevnum
                    },
                };
            },
            "--serverport" => {
                serverport = match arg_pair[1].parse::<u16>() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("port {} is not a number, skipping...", arg_pair[1]);
                        serverport
                    },
                };
            },
            _ => {println!("illegal argument {}, skipping...", arg_pair[0]);},
        }
    }
    (elevnum, serverport)
}
