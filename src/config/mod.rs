use std::path::PathBuf;

use clap::Parser;
use config::{Config as ConfigBuilder, ConfigError, Environment, File};

pub mod types;
pub use types::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Override general.debug setting
    #[arg(long)]
    pub debug: Option<bool>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let args = Args::parse();
        let mut builder = ConfigBuilder::builder();
        builder = builder.add_source(ConfigBuilder::try_from(&Self::default())?);

        // If specific config file is provided via CLI, use only that
        if let Some(config_path) = args.config {
            builder = builder.add_source(File::from(config_path).required(true));
        } else {
            // Otherwise, try to load config files in order of preference
            // Later files override earlier ones
            let config_files = [
                ("config", true),        // config.{json,toml,yaml} - required
                ("config.local", false), // config.local.{json,toml,yaml} - optional
            ];

            // Load from ~/.config/limbo
            let config_dir = dirs::config_dir()
                .map(|p| p.join("limbo"))
                .or_else(|| dirs::home_dir().map(|p| p.join(".config").join("limbo")))
                .ok_or_else(|| {
                    ConfigError::Message("Could not determine config directory".into())
                })?;

            for (base_name, required) in config_files {
                // Try each format
                for ext in ["json", "toml", "yaml", "yml"] {
                    let path = config_dir.join(format!("{}.{}", base_name, ext));
                    if path.exists() {
                        builder = builder.add_source(File::from(path).required(required));
                        break; // Use the first format found for each base name
                    }
                }
            }
        }

        // Load from environment variables with LIMBO_ prefix
        // This will parse nested values like LIMBO_GENERAL__DEBUG=true
        builder = builder.add_source(
            Environment::with_prefix("LIMBO")
                .separator("__") // Use double underscore for nested fields
                .try_parsing(true), // Parse strings to appropriate types
        );

        // Apply command-line overrides
        if let Some(debug) = args.debug {
            builder = builder.set_override("general.debug", debug)?;
        }

        builder.build()?.try_deserialize()
    }
}
