extern crate toml;

use crate::io_tools;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use crate::database::DBManager;


#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub db_config: String,
    pub bind_address: String,
    pub logsize: usize,
}

pub static DEFAULT_CONFIG_PATH: &str = "config.toml";

/// Reads `config.toml` and returns Result with Users on Ok()
///
/// # Examples
///
/// ```rust
/// use transafe_server::config::{read_config, Config};
/// let users = read_config::<Config>("config.toml").unwrap();
/// ```
pub fn read_config<T: Serialize + DeserializeOwned + Clone>(conf_path: &str) -> Result<T, String>
{
    if !io_tools::exists(conf_path) {
        panic!("No `config.toml` file, run `$ webify --setup` ");
    }
    let config_str = match io_tools::read_str(conf_path) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Error on reading the config: {:?}", err);
            return Err("Error on reading the config".to_string());
        }
    };
    let config: T = match toml::from_str(&config_str) {
        Ok(value) => value,
        Err(err) => {
            println!("Something goes wrong while reading the users: {}", err);
            return Err(format!("{:?}", err));
        }
    };

    Ok(config)
}


/// Writes Config to the `config.toml`, returns Result
///
/// # Examples
///
/// ```rust
/// use transafe_server::config::Config;
/// let config = Config {
///     db_config: String::from("database.db"),
///     bind_address: String::from("127.0.0.1:2280"),
///     logsize: 10
/// };
/// write_database(config).unwrap();
/// ```
pub fn write_config<T: Serialize + DeserializeOwned + Clone>(config: T, conf_path: &str) -> Result<(), String> {
    let conf_str = match toml::to_string(&config) {
        Ok(value) => value,
        Err(err) => {
            println!("Something went wrong while parsing the config: {}", err);
            panic!("{}", err);
        }
    };


    match io_tools::write_to_file(conf_path, conf_str) {
        Ok(_) => return Ok(()),
        Err(err) => {
            println!("An error occured while writing to the config: {}", err);
            return Err(format!("{:?}", err));
        }
    };
}

/// Asks all necessary data for configuring the server and writes proper config
pub fn setup() {
    let bind_address = io_tools::read_std_line("Enter address to bind on: ");
    let db_config = io_tools::read_std_line("Enter sqlite path: ");
    let loggersize = io_tools::read_std_line("Enter max logging size: ").parse::<usize>().unwrap();

    match write_config::<Config>(Config {
        db_config: db_config.clone(),
        bind_address: bind_address.clone(),
        logsize: loggersize,
    }, DEFAULT_CONFIG_PATH) {
        Ok(_) => println!("Ok"),
        Err(err) => panic!("{:?}", err),
    };

    match DBManager::init(&db_config) {
        Ok(_) => println!("Ok"),
        Err(err) => panic!("{:?}", err),
    };
}

