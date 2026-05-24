// Copyright (c) 2026 NewtonEmu Contributors
// Licensed under GPL v3

//! Configuration file parsing and management
//!
//! This module handles loading and parsing JSON configuration files
//! that define the emulator's hardware configuration, display settings,
//! storage devices, network configuration, and debugging options.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Complete emulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    #[serde(default)]
    pub cpu: CpuConfig,
    
    #[serde(default)]
    pub memory: MemoryConfig,
    
    #[serde(default)]
    pub display: DisplayConfig,
    
    #[serde(default)]
    pub storage: StorageConfig,
    
    #[serde(default)]
    pub network: NetworkConfig,
    
    #[serde(default)]
    pub debug: DebugConfig,
}

/// CPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuConfig {
    /// CPU model (e.g., "G4_7400", "G4_7450")
    #[serde(default = "default_cpu_model")]
    pub model: String,
    
    /// Clock speed in MHz
    #[serde(default = "default_clock_speed")]
    pub clock_speed: u32,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// RAM size in megabytes
    #[serde(default = "default_ram_size")]
    pub ram_size_mb: usize,
    
    /// Path to ROM file
    #[serde(default)]
    pub rom_path: Option<PathBuf>,
}

/// Display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Display width in pixels
    #[serde(default = "default_display_width")]
    pub width: u32,
    
    /// Display height in pixels
    #[serde(default = "default_display_height")]
    pub height: u32,
    
    /// Color depth in bits per pixel
    #[serde(default = "default_color_depth")]
    pub color_depth: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Boot CD/DVD ISO path
    #[serde(default)]
    pub boot_cd: Option<PathBuf>,
    
    /// Boot disk image path
    #[serde(default)]
    pub boot_disk: Option<PathBuf>,
    
    /// SCSI devices
    #[serde(default)]
    pub scsi: Vec<ScsiDevice>,
    
    /// IDE devices
    #[serde(default)]
    pub ide: Vec<IdeDevice>,
}

/// SCSI device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScsiDevice {
    /// SCSI ID (0-6, 7 is controller)
    pub id: u8,
    
    /// Path to disk image or ISO
    pub path: PathBuf,
    
    /// Read-only flag
    #[serde(default)]
    pub readonly: bool,
}

/// IDE device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeDevice {
    /// IDE channel (0 = primary, 1 = secondary)
    pub channel: u8,
    
    /// Device on channel (0 = master, 1 = slave)
    pub device: u8,
    
    /// Path to disk image
    pub path: PathBuf,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable networking
    #[serde(default)]
    pub enabled: bool,
    
    /// Network type (e.g., "slirp", "tap", "user")
    #[serde(default, rename = "type")]
    pub network_type: Option<String>,
    
    /// MAC address
    #[serde(default)]
    pub mac_address: Option<String>,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// IPC socket path for external debugger/tools
    #[serde(default)]
    pub ipc_socket: Option<PathBuf>,
    
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

// Default values
fn default_cpu_model() -> String {
    "G4_7400".to_string()
}

fn default_clock_speed() -> u32 {
    450 // MHz
}

fn default_ram_size() -> usize {
    256 // MB
}

fn default_display_width() -> u32 {
    800
}

fn default_display_height() -> u32 {
    600
}

fn default_color_depth() -> u32 {
    32
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            cpu: CpuConfig::default(),
            memory: MemoryConfig::default(),
            display: DisplayConfig::default(),
            storage: StorageConfig::default(),
            network: NetworkConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

impl Default for CpuConfig {
    fn default() -> Self {
        Self {
            model: default_cpu_model(),
            clock_speed: default_clock_speed(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            ram_size_mb: default_ram_size(),
            rom_path: None,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: default_display_width(),
            height: default_display_height(),
            color_depth: default_color_depth(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            boot_cd: None,
            boot_disk: None,
            scsi: Vec::new(),
            ide: Vec::new(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            network_type: None,
            mac_address: None,
        }
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            ipc_socket: None,
            log_level: default_log_level(),
        }
    }
}

impl EmulatorConfig {
    /// Get the default configuration file path
    /// Returns ~/.config/NewtonEmu/config.json on Unix-like systems
    /// Returns %APPDATA%\NewtonEmu\config.json on Windows
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\NewtonEmu
            let appdata = std::env::var("APPDATA")
                .context("APPDATA environment variable not found")?;
            PathBuf::from(appdata).join("NewtonEmu")
        } else {
            // Unix-like: ~/.config/NewtonEmu
            let home = std::env::var("HOME")
                .context("HOME environment variable not found")?;
            PathBuf::from(home).join(".config").join("NewtonEmu")
        };
        
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {}", config_dir.display()))?;
        }
        
        Ok(config_dir.join("config.json"))
    }
    
    /// Load configuration from the default location, or return default config if not found
    pub fn load_or_default() -> Result<Self> {
        let default_path = Self::default_config_path()?;
        
        if default_path.exists() {
            Self::load_from_file(&default_path)
        } else {
            tracing::info!("No config file found at {}, using defaults", default_path.display());
            Ok(Self::default())
        }
    }
    
    /// Save configuration to the default location
    pub fn save_default(&self) -> Result<()> {
        let default_path = Self::default_config_path()?;
        self.save_to_file(default_path)
    }
    
    /// Load configuration from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: EmulatorConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON config: {}", path.display()))?;
        
        tracing::info!("Loaded configuration from: {}", path.display());
        tracing::debug!("Config: {:#?}", config);
        
        Ok(config)
    }
    
    /// Save configuration to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize config to JSON")?;
        
        std::fs::write(path, json)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        tracing::info!("Saved configuration to: {}", path.display());
        Ok(())
    }
    
    /// Merge CLI overrides into the configuration
    pub fn apply_cli_overrides(&mut self, 
        cd: Option<PathBuf>,
        disk: Option<PathBuf>,
        rom: Option<PathBuf>,
        headless: bool,
    ) {
        // Override boot CD if specified
        if let Some(cd_path) = cd {
            tracing::info!("CLI override: boot_cd = {}", cd_path.display());
            self.storage.boot_cd = Some(cd_path);
        }
        
        // Override boot disk if specified
        if let Some(disk_path) = disk {
            tracing::info!("CLI override: boot_disk = {}", disk_path.display());
            self.storage.boot_disk = Some(disk_path);
        }
        
        // Override ROM if specified
        if let Some(rom_path) = rom {
            tracing::info!("CLI override: rom_path = {}", rom_path.display());
            self.memory.rom_path = Some(rom_path);
        }
        
        // Headless mode (might want to disable display in the future)
        if headless {
            tracing::info!("CLI override: headless mode enabled");
            // For now, headless just means no window - display still renders
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = EmulatorConfig::default();
        assert_eq!(config.cpu.model, "G4_7400");
        assert_eq!(config.cpu.clock_speed, 450);
        assert_eq!(config.memory.ram_size_mb, 256);
        assert_eq!(config.display.width, 800);
        assert_eq!(config.display.height, 600);
    }
    
    #[test]
    fn test_json_roundtrip() {
        let config = EmulatorConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EmulatorConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.cpu.model, parsed.cpu.model);
        assert_eq!(config.memory.ram_size_mb, parsed.memory.ram_size_mb);
    }
}
