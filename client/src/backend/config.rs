use bevy::prelude::Resource;
use std::collections::HashMap;
use std::fs::read_to_string;

#[derive(Resource)]
pub struct ClientConfig {
    pub name: String,
}

#[derive(Clone)]
pub struct Config {
    pub server: String,
    pub name: String,
}

impl Config {
    pub fn read(file_path: &str) -> Config {
        let mut settings: HashMap<String, String> = HashMap::new();
        match read_to_string(file_path) {
            Ok(content) => {
                for line in content.lines() {
                    let mut parts = line.split('=');
                    if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                        settings.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read config file {}: {}", file_path, e);
            }
        }
        Config {
            server: settings
                .get("server")
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8080".to_string()),
            name: settings
                .get("name")
                .cloned()
                .unwrap_or_else(|| "UnnamedPlayer".to_string()),
        }
    }
}
