//! Configuration loading and management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Data directory path
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// Logging level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Agent configuration
    #[serde(default)]
    pub agents: AgentsConfig,

    /// Privacy settings
    #[serde(default)]
    pub privacy: PrivacyConfig,

    /// Cloud settings
    #[serde(default)]
    pub cloud: CloudConfig,

    /// Consensus settings
    #[serde(default)]
    pub consensus: ConsensusConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    pub pathos: AgentConfig,
    pub logos: AgentConfig,
    pub ethos: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Model to use for this agent
    #[serde(default)]
    pub model: String,

    /// Whether this agent is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Max tokens for this agent
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
            enabled: true,
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Redact email addresses
    #[serde(default = "default_true")]
    pub redact_emails: bool,

    /// Redact phone numbers
    #[serde(default = "default_true")]
    pub redact_phones: bool,

    /// Redact file paths
    #[serde(default = "default_true")]
    pub redact_paths: bool,

    /// Redact API keys
    #[serde(default = "default_true")]
    pub redact_api_keys: bool,

    /// Redact IP addresses
    #[serde(default = "default_true")]
    pub redact_ips: bool,

    /// Redact SSNs
    #[serde(default = "default_true")]
    pub redact_ssns: bool,

    /// Custom regex patterns to redact
    #[serde(default)]
    pub custom_patterns: Vec<CustomPattern>,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            redact_emails: true,
            redact_phones: true,
            redact_paths: true,
            redact_api_keys: true,
            redact_ips: true,
            redact_ssns: true,
            custom_patterns: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPattern {
    /// Pattern name for tokens (e.g., "EMPLOYEE_ID")
    pub name: String,

    /// Regex pattern
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// Enable cloud escalation
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Cloud API endpoint
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Automatically escalate complex queries
    #[serde(default = "default_true")]
    pub auto_escalate: bool,

    /// Max tokens to process locally before escalating
    #[serde(default = "default_max_local_tokens")]
    pub max_local_tokens: u32,

    /// Require explicit consent for each cloud request
    #[serde(default)]
    pub require_consent: bool,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "api.superinstance.ai".to_string(),
            auto_escalate: true,
            max_local_tokens: 4096,
            require_consent: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum confidence threshold for consensus
    #[serde(default = "default_threshold")]
    pub threshold: f32,

    /// Maximum consensus rounds
    #[serde(default = "default_max_rounds")]
    pub max_rounds: u8,

    /// Agent weights
    #[serde(default)]
    pub weights: ConsensusWeights,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            threshold: 0.85,
            max_rounds: 3,
            weights: ConsensusWeights::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusWeights {
    pub pathos: f32,
    pub logos: f32,
    pub ethos: f32,
}

impl Default for ConsensusWeights {
    fn default() -> Self {
        Self {
            pathos: 0.25,
            logos: 0.45,
            ethos: 0.30,
        }
    }
}

// Default value functions
fn default_data_dir() -> String {
    dirs::home_dir()
        .map(|p| p.join(".superinstance").to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.superinstance".to_string())
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> u32 {
    2048
}

fn default_endpoint() -> String {
    "api.superinstance.ai".to_string()
}

fn default_max_local_tokens() -> u32 {
    4096
}

fn default_threshold() -> f32 {
    0.85
}

fn default_max_rounds() -> u8 {
    3
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            log_level: default_log_level(),
            agents: AgentsConfig::default(),
            privacy: PrivacyConfig::default(),
            cloud: CloudConfig::default(),
            consensus: ConsensusConfig::default(),
        }
    }
}

impl Config {
    /// Get the path to the privacy vault database
    pub fn privacy_vault_path(&self) -> Option<PathBuf> {
        Some(PathBuf::from(&self.data_dir).join("vault.db"))
    }

    /// Get the path to the knowledge database
    pub fn knowledge_db_path(&self) -> PathBuf {
        PathBuf::from(&self.data_dir).join("knowledge.db")
    }

    /// Get the models directory
    pub fn models_dir(&self) -> PathBuf {
        PathBuf::from(&self.data_dir).join("models")
    }
}

/// Load configuration from file or return defaults
pub fn load_config(path: Option<&str>) -> anyhow::Result<Config> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".superinstance")
            .join("config.toml"),
    };

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        // Return defaults if no config file exists
        Ok(Config::default())
    }
}

/// Save configuration to file
pub fn save_config(config: &Config, path: Option<&str>) -> anyhow::Result<()> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".superinstance")
            .join("config.toml"),
    };

    // Ensure directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.consensus.threshold, 0.85);
        assert_eq!(config.consensus.max_rounds, 3);
        assert!(config.privacy.redact_emails);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.consensus.threshold, config.consensus.threshold);
    }
}
