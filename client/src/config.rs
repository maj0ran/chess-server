use std::collections::HashMap;
use std::fs::read_to_string;

pub struct Config {
    pub server: String,
    pub name: String,
}

impl Config {
    pub fn read(file_path: &str) -> Config {
        let mut settings: HashMap<String, String> = HashMap::new();
        for line in read_to_string(file_path).unwrap().lines() {
            let mut parts = line.split('=');
            let key = parts.next().unwrap().to_string();
            let value = parts.next().unwrap().to_string();
            settings.insert(key, value);
        }
        Config {
            server: settings.get("server").unwrap().to_string(),
            name: settings.get("name").unwrap().to_string(),
        }
    }
}
