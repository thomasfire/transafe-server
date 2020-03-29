
use transafe_server::server::run_server;
use transafe_server::config;
use std::env;
use std::sync::{Arc, Mutex};


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--setup" => {
                config::setup();
                return;
            },
            _ => {
                println!("Unknown argument, exiting");
                return;
            }
        }
    }
    let config = Arc::new(Mutex::new(config::read_config::<config::Config>(config::DEFAULT_CONFIG_PATH).unwrap()));
    let _handler = run_server(config);
}
