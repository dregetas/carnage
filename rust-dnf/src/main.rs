use clap::{Parser, Subcommand};
use anyhow::Result;

mod config;
mod repo;
mod package;
mod repo_manager;
mod db;

use crate::config::Config;
use crate::repo_manager::RepositoryManager;
use crate::db::PackageDatabase;

#[derive(Parser)]
#[command(name = "rust-dnf")]
#[command(about = "A DNF alternative written in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install {
        packages: Vec<String>,
    },
    /// Remove packages
    Remove {
        packages: Vec<String>,
    },
    /// Update package database
    Update,
    /// Search for packages
    Search {
        query: String,
    },
    /// List installed packages
    List,
    /// Show package information
    Info {
        package: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    
    // Load configuration
    let config = Config::load()?;
    
    // Initialize repository manager and package database
    let mut repo_manager = RepositoryManager::new(config.clone());
    let mut pkg_db = PackageDatabase::new(config.database_dir.join("packages.json"));
    
    // Load existing data
    repo_manager.load_repositories()?;
    pkg_db.load()?;
    
    match cli.command {
        Commands::Install { packages } => {
            println!("Installing packages: {:?}", packages);
            for pkg_name in packages {
                if let Some(pkg) = repo_manager.find_package(&pkg_name) {
                    println!("Found package: {} {}", pkg.name.name, pkg.version.version);
                    // TODO: Implement actual installation
                    pkg_db.install_package(pkg.clone())?;
                    println!("Package {} installed successfully!", pkg.name.name);
                } else {
                    eprintln!("Package {} not found in repositories", pkg_name);
                }
            }
        }
        Commands::Remove { packages } => {
            println!("Removing packages: {:?}", packages);
            for pkg_name in packages {
                if pkg_db.is_installed(&pkg_name) {
                    pkg_db.remove_package(&pkg_name)?;
                    println!("Package {} removed successfully!", pkg_name);
                } else {
                    eprintln!("Package {} is not installed", pkg_name);
                }
            }
        }
        Commands::Update => {
            println!("Updating package database");
            repo_manager.update()?;
            println!("Repository metadata updated successfully!");
        }
        Commands::Search { query } => {
            println!("Searching for: {}", query);
            let results = repo_manager.search_packages(&query);
            if results.is_empty() {
                println!("No packages found matching '{}'", query);
            } else {
                println!("Found {} packages:", results.len());
                for pkg in results {
                    println!("  {} - {}", pkg.name.name, pkg.summary);
                }
            }
        }
        Commands::List => {
            println!("Listing installed packages:");
            let installed = pkg_db.list_installed();
            if installed.is_empty() {
                println!("No packages installed");
            } else {
                for pkg in installed {
                    println!("  {} {}-{}", 
                        pkg.package.name.name, 
                        pkg.package.version.version,
                        pkg.package.version.release
                    );
                }
            }
        }
        Commands::Info { package } => {
            println!("Showing info for: {}", package);
            if let Some(pkg) = repo_manager.find_package(&package) {
                println!("Package: {}", pkg.name.name);
                println!("Version: {}-{}", pkg.version.version, pkg.version.release);
                println!("Architecture: {}", pkg.name.arch);
                println!("Description: {}", pkg.description);
                println!("Summary: {}", pkg.summary);
            } else {
                eprintln!("Package {} not found", package);
            }
        }
    }
    
    Ok(())
}