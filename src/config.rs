use dirs;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Schedule {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RoomConfig {
    pub name: String,
    pub temp_sensor: String,
    pub trv_device: String,
    pub schedule: Schedule,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct HouseConfig {
    pub rooms: Vec<RoomConfig>,
}

impl HouseConfig {
    pub fn new() -> HouseConfig {
        HouseConfig { rooms: Vec::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub max_packet_size: u32,
    pub house: HouseConfig,
}

impl Config {
    /// Returns the default path to the config file in the XDG config directory.
    fn default_config_path() -> PathBuf {
        let mut path = dirs::config_dir().expect("Unable to find config directory");
        path.push("chinum");
        path.push("config");
        path
    }

    /// Serializes the config to the file at the specified path or the default XDG path.
    #[allow(dead_code)]
    pub fn save(&self) -> io::Result<()> {
        self.save_to(None)
    }

    /// Serializes the config to the file at the given path (used for testing).
    fn save_to(&self, path: Option<PathBuf>) -> io::Result<()> {
        let path = path.unwrap_or_else(|| Self::default_config_path());
        // Create the directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    /// Deserializes the config from the file at the default XDG path.
    pub fn load() -> io::Result<Self> {
        Self::load_from(None)
    }

    /// Deserializes the config from the file at the given path (used for testing).
    fn load_from(path: Option<PathBuf>) -> io::Result<Self> {
        let path = path.unwrap_or_else(|| Self::default_config_path());
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Helper function to set up a temporary config path
    fn setup_temp_config_path(temp_dir: &TempDir) -> PathBuf {
        let mut path = temp_dir.path().to_path_buf();
        path.push("chinum");
        path.push("config");
        path
    }

    #[test]
    fn test_save_and_load_config() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = setup_temp_config_path(&temp_dir);

        // Create a Config instance
        let config = Config {
            server: "test.example.com".to_string(),
            port: 8080,
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            max_packet_size: 52342,
        };

        // Test saving
        assert!(
            config.save_to(Some(config_path.clone())).is_ok(),
            "Failed to save config"
        );

        // Verify the file was created
        assert!(config_path.exists(), "Config file was not created");

        // Test loading
        let loaded_config = Config::load_from(Some(config_path)).expect("Failed to load config");

        // Verify the loaded config matches the original
        assert_eq!(
            config, loaded_config,
            "Loaded config does not match saved config"
        );
    }

    #[test]
    fn test_load_nonexistent_file() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = setup_temp_config_path(&temp_dir);

        // Test loading a nonexistent file
        let result = Config::load_from(Some(config_path));
        assert!(result.is_err(), "Loading nonexistent file should fail");
    }
}
