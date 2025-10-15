use crate::package::Package;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use log;

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub package: Package,
    pub installed_files: Vec<String>,
    pub install_time: String, // ISO timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageDatabase {
    pub installed_packages: HashMap<String, InstalledPackage>,
    pub database_path: PathBuf,
}

impl PackageDatabase {
    pub fn new(database_path: PathBuf) -> Self {
        Self {
            installed_packages: HashMap::new(),
            database_path,
        }
    }
    
    pub fn load(&mut self) -> Result<()> {
        log::info!("Loading package database from {:?}", self.database_path);
        
        if !self.database_path.exists() {
            log::warn!("Package database does not exist, creating new one");
            return Ok(());
        }
        
        let data = fs::read_to_string(&self.database_path)?;
        let db: PackageDatabase = serde_json::from_str(&data)?;
        
        self.installed_packages = db.installed_packages;
        log::info!("Loaded {} installed packages", self.installed_packages.len());
        
        Ok(())
    }
    
    pub fn save(&self) -> Result<()> {
        log::debug!("Saving package database to {:?}", self.database_path);
        
        if let Some(parent) = self.database_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let data = serde_json::to_string_pretty(&self)?;
        fs::write(&self.database_path, data)?;
        
        log::debug!("Package database saved successfully");
        Ok(())
    }
    
    pub fn install_package(&mut self, package: Package) -> Result<()> {
        let package_key = format!("{}.{}", package.name.name, package.name.arch);
        
        let installed_pkg = InstalledPackage {
            package,
            installed_files: Vec::new(), // We'll populate this during actual installation
            install_time: chrono::Utc::now().to_rfc3339(),
        };
        
        self.installed_packages.insert(package_key, installed_pkg);
        self.save()?;
        
        log::info!("Package added to database");
        Ok(())
    }
    
    pub fn remove_package(&mut self, package_name: &str) -> Result<()> {
        if self.installed_packages.remove(package_name).is_some() {
            self.save()?;
            log::info!("Package {} removed from database", package_name);
            Ok(())
        } else {
            anyhow::bail!("Package {} not found in database", package_name)
        }
    }
    
    pub fn list_installed(&self) -> Vec<&InstalledPackage> {
        self.installed_packages.values().collect()
    }
    
    pub fn is_installed(&self, package_name: &str) -> bool {
        self.installed_packages.contains_key(package_name)
    }
    
    pub fn get_installed(&self, package_name: &str) -> Option<&InstalledPackage> {
        self.installed_packages.get(package_name)
    }
}