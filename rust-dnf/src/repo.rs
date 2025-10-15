use crate::package::Package;
use crate::config::Repository as RepoConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use reqwest::blocking::Client;
use flate2::read::GzDecoder;
use std::io::Read;
use quick_xml::events::Event;
use quick_xml::Reader;
use log;

#[derive(Debug)]
pub struct Repository {
    pub config: RepoConfig,
    pub packages: HashMap<String, Package>,
}

impl Repository {
    pub fn new(config: RepoConfig) -> Self {
        Self {
            config,
            packages: HashMap::new(),
        }
    }
    
    pub fn load_metadata(&mut self, cache_dir: &PathBuf) -> Result<()> {
        log::info!("Loading metadata for repository: {}", self.config.name);
        
        // Create repository cache directory
        let repo_cache_dir = cache_dir.join(&self.config.name);
        fs::create_dir_all(&repo_cache_dir)?;
        
        // Try to download real metadata
        if let Err(e) = self.try_download_metadata(&repo_cache_dir) {
            log::warn!("Failed to download real metadata for {}: {}", self.config.name, e);
            log::info!("Falling back to mock data");
            self.load_mock_data()?;
        }
        
        log::debug!("Repository {} metadata loaded with {} packages", 
                   self.config.name, self.packages.len());
        Ok(())
    }

    fn try_download_metadata(&mut self, repo_cache_dir: &PathBuf) -> Result<()> {
        // Try different metadata locations (Fedora uses repomd.xml)
        let metadata_paths = vec![
            "repodata/repomd.xml",
            "repodata/primary.xml.gz",
            "repodata/primary.sqlite.gz",
        ];
        
        for metadata_path in metadata_paths {
            let metadata_url = format!("{}/{}", self.config.url, metadata_path);
            let local_path = repo_cache_dir.join(metadata_path);
            
            if self.download_file(&metadata_url, &local_path).is_ok() {
                log::info!("Successfully downloaded metadata from: {}", metadata_url);
                
                // Parse based on file type
                if metadata_path.ends_with("primary.xml.gz") {
                    return self.parse_primary_xml(&local_path);
                } else if metadata_path.ends_with("repomd.xml") {
                    if let Ok(primary_location) = self.parse_repomd(&local_path) {
                        let primary_url = format!("{}/{}", self.config.url, primary_location);
                        let primary_path = repo_cache_dir.join("primary.xml.gz");
                        
                        if self.download_file(&primary_url, &primary_path).is_ok() {
                            return self.parse_primary_xml(&primary_path);
                        }
                    }
                }
            }
        }
        
        anyhow::bail!("Could not download or parse any metadata files")
    }
    
    fn parse_repomd(&self, path: &PathBuf) -> Result<String> {
        let content = fs::read_to_string(path)?;
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut in_data = false;
        let mut data_type = String::new();
        let mut location = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"data" => {
                            in_data = true;
                            if let Some(typ) = e.try_get_attribute("type")? {
                                data_type = String::from_utf8_lossy(&typ.value).to_string();
                            }
                        }
                        b"location" if in_data && data_type == "primary" => {
                            if let Some(href) = e.try_get_attribute("href")? {
                                location = String::from_utf8_lossy(&href.value).to_string();
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().as_ref() == b"data" {
                        in_data = false;
                        if !location.is_empty() {
                            return Ok(location);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::warn!("Error parsing repomd.xml: {}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        
        anyhow::bail!("Could not find primary metadata location in repomd.xml")
    }
    
    fn parse_primary_xml(&mut self, path: &PathBuf) -> Result<()> {
        log::info!("Parsing primary metadata from: {:?}", path);
        
        // Decompress if needed
        let mut file = fs::File::open(path)?;
        let mut content = String::new();
        
        if path.extension().map(|ext| ext == "gz").unwrap_or(false) {
            let mut decoder = GzDecoder::new(file);
            decoder.read_to_string(&mut content)?;
        } else {
            file.read_to_string(&mut content)?;
        }
        
        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        
        let mut buf = Vec::new();
        let mut current_package: Option<Package> = None;
        let mut current_text = String::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    current_text.clear();
                    
                    if e.name().as_ref() == b"package" {
                        current_package = Some(Package::new(
                            crate::package::PackageName::new("unknown", "x86_64").unwrap(),
                            crate::package::Version::new(0, "0", "0").unwrap(),
                            String::new(),
                        ));
                    }
                }
                
                Ok(Event::Text(e)) => {
                    current_text.push_str(&e.unescape()?);
                }
                
                Ok(Event::End(e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    if let Some(ref mut pkg) = current_package {
                        match tag.as_str() {
                            "name" => {
                                pkg.name = crate::package::PackageName::new(&current_text, &self.config.name).unwrap();
                            }
                            "arch" => {
                                pkg.name.arch = current_text.clone();
                            }
                            "version" => {
                                // Version parsing will be handled by attributes
                            }
                            "summary" => {
                                pkg.summary = current_text.clone();
                            }
                            "description" => {
                                pkg.description = current_text.clone();
                            }
                            "package" => {
                                // End of package - add to hashmap
                                if pkg.name.name != "unknown" {
                                    self.packages.insert(pkg.name.name.clone(), pkg.clone());
                                }
                                current_package = None;
                            }
                            _ => {}
                        }
                    }
                    
                    // Handle version with attributes
                    if tag == "version" {
                        if let Some(ref mut pkg) = current_package {
                            // For now, use a simple version - we'll parse attributes later
                            pkg.version = crate::package::Version::parse(&current_text).unwrap();
                        }
                    }
                }
                
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::warn!("XML parsing error: {}, continuing...", e);
                    continue;
                }
                _ => {}
            }
            buf.clear();
        }
        
        log::info!("Parsed {} packages from primary metadata", self.packages.len());
        Ok(())
    }
    
    fn download_file(&self, url: &str, path: &PathBuf) -> Result<()> {
        log::debug!("Downloading {} to {:?}", url, path);
        
        let client = Client::new();
        let response = client.get(url).send()?;
        
        if response.status().is_success() {
            let content = response.bytes()?;
            fs::write(path, content)?;
            log::debug!("Successfully downloaded {}", url);
            Ok(())
        } else {
            anyhow::bail!("Failed to download {}: {}", url, response.status());
        }
    }

    fn load_mock_data(&mut self) -> Result<()> {
        log::warn!("Using mock data for repository: {}", self.config.name);
        
        let mock_packages = vec![
            Package::new(
                crate::package::PackageName::new("nano", "x86_64").unwrap(),
                crate::package::Version::new(0, "2.9.8", "1.fc39").unwrap(),
                "A small text editor for consoles".to_string(),
            ),
            Package::new(
                crate::package::PackageName::new("vim", "x86_64").unwrap(),
                crate::package::Version::new(0, "8.2", "1.fc39").unwrap(),
                "Vi Improved - enhanced vi editor".to_string(),
            ),
            Package::new(
                crate::package::PackageName::new("curl", "x86_64").unwrap(),
                crate::package::Version::new(0, "7.61.1", "1.fc39").unwrap(),
                "Tool for transferring data with URL syntax".to_string(),
            ),
            Package::new(
                crate::package::PackageName::new("rust", "x86_64").unwrap(),
                crate::package::Version::new(0, "1.70.0", "1.fc39").unwrap(),
                "The Rust programming language".to_string(),
            ),
            Package::new(
                crate::package::PackageName::new("firefox", "x86_64").unwrap(),
                crate::package::Version::new(0, "115.0", "1.fc39").unwrap(),
                "Mozilla Firefox Web browser".to_string(),
            ),
            Package::new(
                crate::package::PackageName::new("git", "x86_64").unwrap(),
                crate::package::Version::new(0, "2.43.0", "1.fc39").unwrap(),
                "Fast Version Control System".to_string(),
            ),
            Package::new(
                crate::package::PackageName::new("python3", "x86_64").unwrap(),
                crate::package::Version::new(0, "3.11.5", "1.fc39").unwrap(),
                "Python programming language".to_string(),
            ),
        ];
        
        for pkg in mock_packages {
            self.packages.insert(pkg.name.name.clone(), pkg);
        }
        
        Ok(())
    }
    
    pub fn find_package(&self, name: &str) -> Option<&Package> {
        self.packages.get(name)
    }
    
    pub fn search(&self, query: &str) -> Vec<&Package> {
        let query_lower = query.to_lowercase();
        self.packages
            .values()
            .filter(|pkg| {
                pkg.name.name.to_lowercase().contains(&query_lower) || 
                pkg.description.to_lowercase().contains(&query_lower) ||
                pkg.summary.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
    
    pub fn list_packages(&self) -> Vec<&Package> {
        self.packages.values().collect()
    }
}