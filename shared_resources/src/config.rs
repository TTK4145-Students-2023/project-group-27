use std::fs;
use std::collections::HashMap;
use std::env;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct HRAConfigFile {
    exec_folder_path: String,
    operating_systems: HashMap<String, String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConfigFile {
    pub network: HashMap<String, Vec<u16>>,
    pub server: HashMap<String, u16>,
    pub elevator: HashMap<String, u8>,
    pub hall_request_assigner: HRAConfigFile,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub update_port: u16,
    pub command_port: u16,
    pub pp_update_port: u16,
    pub pp_ack_port: u16
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

fn parse_env_args(defaultport: u16) -> (u8, u16) {
    let (mut num, mut serverport) = (0, defaultport);

    let args: Vec<String> = env::args().collect();
    for arg_pair in args.rchunks_exact(2) {
        match arg_pair[0].as_str() {
            "--num" => {
                num = match arg_pair[1].parse::<u8>() {
                    Ok(num) => num,
                    Err(_) => {
                        println!("num {} is not a number, skipping...", arg_pair[1]);
                        num
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
    (num, serverport)
}

#[derive(Debug, Clone)]
pub struct SlaveConfig {
    pub elevnum: u8,
    pub network: NetworkConfig,
    pub server: ServerConfig,
    pub elevator: ElevatorConfig,
}

impl SlaveConfig {
    pub fn get() -> Self {
        let config_file = read_config_file().unwrap();
        let (elevnum, serverport) = parse_env_args(config_file.server["port"]);
        
        SlaveConfig {
            elevnum: elevnum,
            network: NetworkConfig { 
                update_port: config_file.network["update_ports"][elevnum as usize], 
                command_port: config_file.network["command_ports"][elevnum as usize],
                pp_update_port: config_file.network["slave_pp_update_ports"][elevnum as usize],
                pp_ack_port: config_file.network["slave_pp_ack_ports"][elevnum as usize]
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
    pub backup_update_port: u16,
    pub backup_ack_port: u16,
    pub pp_port: u16,
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
        let exec_path = config_file.hall_request_assigner.exec_folder_path.clone()
            + &config_file.hall_request_assigner.operating_systems[env::consts::OS];

        MasterConfig {
            network: MasterNetworkConfig { 
                update_ports: config_file.network["update_ports"].to_vec(),
                command_ports: config_file.network["command_ports"].to_vec(),
                backup_update_port: config_file.network["backup_update_ports"][0],
                backup_ack_port: config_file.network["backup_ack_ports"][0],
                pp_port: config_file.network["master_pp_ports"][0],
            },
            elevator: ElevatorConfig { 
                num_floors: config_file.elevator["num_floors"], 
            },
            hall_request_assigner: HallRequestAssignerConfig { 
                exec_path: exec_path,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupNetworkConfig {
    pub backup_update_port: u16,
    pub backup_ack_port: u16,
    pub pp_port: u16,
}

#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub network: BackupNetworkConfig,
    pub elevator: ElevatorConfig,
}

impl BackupConfig {
    pub fn get() -> Self {
        let config_file = read_config_file().unwrap();
        BackupConfig {
            network: BackupNetworkConfig { 
                backup_update_port: config_file.network["backup_update_ports"][0],
                backup_ack_port: config_file.network["backup_ack_ports"][0],
                pp_port: config_file.network["backup_pp_ports"][0],
            },
            elevator: ElevatorConfig { 
                num_floors: config_file.elevator["num_floors"], 
            },
        }
    }
}
