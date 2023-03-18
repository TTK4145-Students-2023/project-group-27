use std::fs;
use std::collections::HashMap;
use std::env;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfigFile {
    pub network: HashMap<String, Vec<u16>>,
    pub server: HashMap<String, u16>,
    pub elevator: HashMap<String, u8>,
    pub hall_request_assigner: HashMap<String, String>,
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
pub struct ElevatorConfig {
    pub num_floors: u8,
}

fn read_config_file() -> Result<ConfigFile, serde_json::Error> {
    let file_path = "../config.json";
    let fallback_file_path = "../_config.json";
    let config_contents = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            println!("No configuration file provided, using default settings...");
            fs::read_to_string(fallback_file_path).unwrap()
        },
    };
    serde_json::from_str(&config_contents)
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

#[derive(Debug, Clone)]
pub struct SlaveConfig {
    pub network: NetworkConfig,
    pub server: ServerConfig,
    pub elevator: ElevatorConfig,
}

impl SlaveConfig {
    pub fn get() -> Self {
        let config_file = read_config_file().unwrap();
        let (elevnum, serverport) = parse_env_args(config_file.server["port"]);
        
        SlaveConfig {
            network: NetworkConfig { 
                update_port: config_file.network["update_ports"][elevnum as usize], 
                command_port: config_file.network["command_ports"][elevnum as usize],
            },
            server: ServerConfig { 
                port: serverport,
            },
            elevator: ElevatorConfig { 
                num_floors: config_file.elevator["num_floors"], 
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct MasterNetworkConfig {
    pub update_ports: Vec<u16>,
    pub command_ports: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct HallRequestAssignerConfig {
    pub exec_path: String,
}

#[derive(Debug, Clone)]
pub struct MasterConfig {
    pub network: MasterNetworkConfig,
    pub elevator: ElevatorConfig,
    pub hall_request_assigner: HallRequestAssignerConfig,
}

impl MasterConfig {
    pub fn get() -> Self {
        let config_file = read_config_file().unwrap();
        
        MasterConfig {
            network: MasterNetworkConfig { 
                update_ports: config_file.network["update_ports"].to_vec(),
                command_ports: config_file.network["command_ports"].to_vec(),
            },
            elevator: ElevatorConfig { 
                num_floors: config_file.elevator["num_floors"], 
            },
            hall_request_assigner: HallRequestAssignerConfig { 
                exec_path: config_file.hall_request_assigner["exec_path"].clone(),
            }
        }
    }
}
