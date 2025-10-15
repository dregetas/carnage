use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub gpg_check: bool,
    pub gpg_key: Option<String>,
    pub metadata_sig: bool,  // Whether to check metadata signatures
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub repositories: HashMap<String, Repository>,
    pub cache_dir: PathBuf,
    pub install_root: PathBuf,
    pub database_dir: PathBuf,
    pub releasever: String,  // Fedora release version
    pub basearch: String,    // Base architecture
}

impl Default for Config {
    fn default() -> Self {
        let mut repos = HashMap::new();
        
        // Fedora 39 repositories
        let releasever = "39";
        let basearch = "x86_64";
        
        repos.insert(
            "fedora".to_string(),
            Repository {
                name: "fedora".to_string(),
                url: format!("https://download.fedoraproject.org/pub/fedora/linux/releases/{}/Everything/{}/os/", releasever, basearch),
                enabled: true,
                gpg_check: true,
                gpg_key: Some("/etc/pki/rpm-gpg/RPM-GPG-KEY-fedora-39-x86_64".to_string()),
                metadata_sig: true,
            },
        );
        
        repos.insert(
            "updates".to_string(),
            Repository {
                name: "updates".to_string(),
                url: format!("https://download.fedoraproject.org/pub/fedora/linux/updates/{}/Everything/{}/", releasever, basearch),
                enabled: true,
                gpg_check: true,
                gpg_key: Some("/etc/pki/rpm-gpg/RPM-GPG-KEY-fedora-39-x86_64".to_string()),
                metadata_sig: true,
            },
        );
        
        repos.insert(
            "fedora-modular".to_string(),
            Repository {
                name: "fedora-modular".to_string(),
                url: format!("https://download.fedoraproject.org/pub/fedora/linux/releases/{}/Modular/{}/os/", releasever, basearch),
                enabled: true,
                gpg_check: true,
                gpg_key: Some("/etc/pki/rpm-gpg/RPM-GPG-KEY-fedora-39-x86_64".to_string()),
                metadata_sig: true,
            },
        );

        Self {
            repositories: repos,
            cache_dir: PathBuf::from("/var/cache/rust-dnf"),
            install_root: PathBuf::from("/"),
            database_dir: PathBuf::from("/var/lib/rust-dnf"),
            releasever: releasever.to_string(),
            basearch: basearch.to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        // For now, just return the default config
        let config = Config::default();
        
        // Create necessary directories
        fs::create_dir_all(&config.cache_dir)?;
        fs::create_dir_all(&config.database_dir)?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = "/etc/rust-dnf/config.toml";
        let config_dir = std::path::Path::new(config_path).parent().unwrap();
        
        fs::create_dir_all(config_dir)?;
        let config_toml = toml::to_string_pretty(self)?;
        fs::write(config_path, config_toml)?;
        
        Ok(())
    }
}