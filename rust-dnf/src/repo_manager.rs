use crate::config::Config;
use crate::repo::Repository as Repo;
use anyhow::Result;
use std::collections::HashMap;
use log;

#[derive(Debug)]
pub struct RepositoryManager {
    pub repositories: HashMap<String, Repo>,
    pub config: Config,
}

impl RepositoryManager {
    pub fn new(config: Config) -> Self {
        Self {
            repositories: HashMap::new(),
            config,
        }
    }
    
    pub fn load_repositories(&mut self) -> Result<()> {
        log::info!("Loading repositories");
        
        for (name, repo_config) in &self.config.repositories {
            if !repo_config.enabled {
                log::debug!("Skipping disabled repository: {}", name);
                continue;
            }
            
            log::info!("Loading repository: {}", name);
            let mut repo = Repo::new(repo_config.clone());
            repo.load_metadata(&self.config.cache_dir)?;
            
            self.repositories.insert(name.clone(), repo);
        }
        
        log::info!("Loaded {} repositories", self.repositories.len());
        Ok(())
    }
    
    pub fn find_package(&self, package_name: &str) -> Option<&crate::package::Package> {
        for repo in self.repositories.values() {
            if let Some(pkg) = repo.find_package(package_name) {
                return Some(pkg);
            }
        }
        None
    }
    
    pub fn search_packages(&self, query: &str) -> Vec<&crate::package::Package> {
        let mut results = Vec::new();
        
        for repo in self.repositories.values() {
            results.extend(repo.search(query));
        }
        
        results.sort_by(|a, b| a.name.name.cmp(&b.name.name));
        results
    }
    
    pub fn update(&mut self) -> Result<()> {
        log::info!("Updating repository metadata");
        self.repositories.clear();
        self.load_repositories()
    }
}